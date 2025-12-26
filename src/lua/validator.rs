//! Lua code validator for safe execution
//!
//! Validates Lua code before execution to:
//! 1. Check syntax errors
//! 2. Detect forbidden functions (io, os, debug, etc.)
//! 3. Provide LLM-friendly error messages with suggestions

use crate::lua::errors::{
    available_functions, blocked_functions, ErrorType, FunctionInfo, ValidationError,
    ValidationResult, ValidationWarning,
};
use regex::Regex;
use rlua::{Lua, Result as LuaResult};
use std::collections::HashSet;

/// Lua code validator
pub struct LuaValidator {
    /// Set of blocked function patterns
    blocked_patterns: Vec<(Regex, String)>,
    /// Set of available function names (for suggestions)
    available_functions: HashSet<String>,
    /// Function info for help
    function_info: Vec<FunctionInfo>,
}

impl Default for LuaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaValidator {
    /// Create a new validator
    pub fn new() -> Self {
        let mut blocked_patterns = Vec::new();

        for (pattern, suggestion) in blocked_functions() {
            // Create regex to match function calls like io.open, os.execute, etc.
            let regex_pattern = pattern.replace(".", r"\.");
            if let Ok(regex) = Regex::new(&format!(r"\b{}\s*\(", regex_pattern)) {
                blocked_patterns.push((regex, suggestion.to_string()));
            }
            // Also match the module itself (e.g., "io" or "os" standalone)
            if pattern.contains('.') {
                let module = pattern.split('.').next().unwrap();
                if let Ok(regex) = Regex::new(&format!(r"\b{}\.", module)) {
                    blocked_patterns.push((regex, format!("The '{}' module is not available. {}", module, suggestion)));
                }
            }
        }

        let function_info = available_functions();
        let available_functions: HashSet<String> =
            function_info.iter().map(|f| f.name.clone()).collect();

        Self {
            blocked_patterns,
            available_functions,
            function_info,
        }
    }

    /// Validate Lua code without executing it
    pub fn validate(&self, code: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Step 1: Check for forbidden functions
        for (pattern, suggestion) in &self.blocked_patterns {
            if let Some(m) = pattern.find(code) {
                let line = Self::line_number(code, m.start());
                let snippet = Self::extract_snippet(code, m.start(), 40);

                let error = ValidationError {
                    error_type: ErrorType::ForbiddenFunction,
                    message: format!("Forbidden function call detected: '{}'", m.as_str().trim_end_matches('(')),
                    line: Some(line),
                    column: None,
                    suggestion: suggestion.clone(),
                    code_snippet: Some(snippet),
                };
                errors.push(error);
            }
        }

        // Step 2: Check syntax by trying to parse
        if let Err(syntax_error) = self.check_syntax(code) {
            let (message, line) = Self::parse_lua_error(&syntax_error);
            let suggestion = self.suggest_fix(&message, code);

            let error = ValidationError {
                error_type: ErrorType::SyntaxError,
                message,
                line,
                column: None,
                suggestion,
                code_snippet: line.map(|l| Self::get_line(code, l)),
            };
            errors.push(error);
        }

        // Step 3: Check for missing return (warning only)
        if !self.has_return_statement(code) && !code.trim().is_empty() {
            warnings.push(ValidationWarning::missing_return());
        }

        // Build result
        if errors.is_empty() {
            let mut result = ValidationResult::valid().with_functions(self.function_info.clone());
            result.warnings = warnings;
            result
        } else {
            let mut result = ValidationResult::invalid(errors);
            result.warnings = warnings;
            result.available_functions = self.function_info.clone();
            result
        }
    }

    /// Check Lua syntax by parsing the code
    fn check_syntax(&self, code: &str) -> LuaResult<()> {
        let lua = Lua::new();
        // Just try to load/parse, don't execute
        lua.load(code).into_function()?;
        Ok(())
    }

    /// Parse rlua error message to extract useful info
    fn parse_lua_error(error: &rlua::Error) -> (String, Option<usize>) {
        let error_str = error.to_string();

        // Try to extract line number from error like "[string "..."]:3: ..."
        let line_regex = Regex::new(r"\]:(\d+):").ok();
        let line = line_regex.and_then(|re| {
            re.captures(&error_str)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse().ok())
        });

        // Clean up the error message
        let message = error_str
            .replace("[string \"...\"]:", "Line ")
            .replace("[string \"<eval>\"]:", "Line ");

        (message, line)
    }

    /// Suggest a fix based on the error message
    fn suggest_fix(&self, error_message: &str, code: &str) -> String {
        let error_lower = error_message.to_lowercase();

        // Common error patterns and suggestions
        if error_lower.contains("unexpected symbol near") {
            if error_lower.contains("'='") {
                return "Check for assignment vs comparison: use '=' for assignment, '==' for comparison.".to_string();
            }
            if error_lower.contains("')'") {
                return "Check for matching parentheses. You may have an extra ')'.".to_string();
            }
            if error_lower.contains("'('") {
                return "Check for matching parentheses. You may be missing '(' or have an extra one.".to_string();
            }
            return "Check the syntax around the unexpected symbol.".to_string();
        }

        if error_lower.contains("'end' expected") {
            return "Add 'end' to close your function, if, for, or while block.".to_string();
        }

        if error_lower.contains("'then' expected") {
            return "Add 'then' after your 'if' condition.".to_string();
        }

        if error_lower.contains("'do' expected") {
            return "Add 'do' after your 'for' or 'while' statement.".to_string();
        }

        if error_lower.contains("unfinished string") {
            return "Close your string with a matching quote (' or \").".to_string();
        }

        if error_lower.contains("attempt to call") {
            // Try to find similar function name
            if let Some(func_name) = self.extract_function_name(code, error_message) {
                if let Some(suggestion) = self.find_similar_function(&func_name) {
                    return format!("Did you mean '{}'? Check the function name.", suggestion);
                }
            }
            return "Check that the function exists and is spelled correctly.".to_string();
        }

        // Default suggestion
        "Review the Lua syntax. Ensure all blocks are properly closed and punctuation is correct.".to_string()
    }

    /// Check if code has a return statement
    fn has_return_statement(&self, code: &str) -> bool {
        // Simple check - look for return keyword not in comments
        let lines: Vec<&str> = code.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }
            // Remove inline comments
            let code_part = if let Some(idx) = trimmed.find("--") {
                &trimmed[..idx]
            } else {
                trimmed
            };
            if code_part.contains("return") {
                return true;
            }
        }
        false
    }

    /// Get line number from character position
    fn line_number(code: &str, pos: usize) -> usize {
        code[..pos].chars().filter(|c| *c == '\n').count() + 1
    }

    /// Extract a code snippet around a position
    fn extract_snippet(code: &str, pos: usize, context: usize) -> String {
        let start = pos.saturating_sub(context);
        let end = (pos + context).min(code.len());
        let snippet = &code[start..end];

        // Clean up and add ellipsis if truncated
        let mut result = String::new();
        if start > 0 {
            result.push_str("...");
        }
        result.push_str(snippet.trim());
        if end < code.len() {
            result.push_str("...");
        }
        result
    }

    /// Get a specific line from code
    fn get_line(code: &str, line_num: usize) -> String {
        code.lines()
            .nth(line_num.saturating_sub(1))
            .unwrap_or("")
            .to_string()
    }

    /// Try to extract function name from error context
    fn extract_function_name(&self, _code: &str, error: &str) -> Option<String> {
        // Try to extract from "attempt to call a nil value (global 'xyz')"
        let re = Regex::new(r"global '(\w+)'").ok()?;
        re.captures(error)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Find a similar function name (for "did you mean" suggestions)
    fn find_similar_function(&self, name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();

        // Simple similarity: look for functions that start with same letters
        // or have small edit distance
        for func in &self.available_functions {
            let func_lower = func.to_lowercase();
            if func_lower.starts_with(&name_lower[..name_lower.len().min(3).max(1)]) {
                return Some(func.clone());
            }
            if Self::levenshtein(&name_lower, &func_lower) <= 2 {
                return Some(func.clone());
            }
        }
        None
    }

    /// Simple Levenshtein distance
    fn levenshtein(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let m = a_chars.len();
        let n = b_chars.len();

        if m == 0 {
            return n;
        }
        if n == 0 {
            return m;
        }

        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 0..=m {
            dp[i][0] = i;
        }
        for j in 0..=n {
            dp[0][j] = j;
        }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[m][n]
    }

    /// Get available functions info (for LLM reference)
    pub fn get_available_functions(&self) -> &[FunctionInfo] {
        &self.function_info
    }

    /// Format available functions as a help string
    pub fn format_help(&self) -> String {
        let mut help = String::from("Available Liath functions:\n\n");

        for func in &self.function_info {
            help.push_str(&format!("  {} - {}\n", func.signature, func.description));
            if let Some(example) = &func.example {
                help.push_str(&format!("    Example: {}\n", example));
            }
            help.push('\n');
        }

        help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_code() {
        let validator = LuaValidator::new();
        let result = validator.validate("return 1 + 1");
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_syntax_error() {
        let validator = LuaValidator::new();
        let result = validator.validate("return 1 +");
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].error_type, ErrorType::SyntaxError);
    }

    #[test]
    fn test_forbidden_function_io() {
        let validator = LuaValidator::new();
        let result = validator.validate("io.open('/etc/passwd')");
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].error_type, ErrorType::ForbiddenFunction);
        assert!(result.errors[0].suggestion.contains("put"));
    }

    #[test]
    fn test_forbidden_function_os() {
        let validator = LuaValidator::new();
        let result = validator.validate("os.execute('rm -rf /')");
        assert!(!result.valid);
        assert_eq!(result.errors[0].error_type, ErrorType::ForbiddenFunction);
    }

    #[test]
    fn test_missing_return_warning() {
        let validator = LuaValidator::new();
        let result = validator.validate("local x = 1");
        // Should be valid but with warning
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(LuaValidator::levenshtein("put", "put"), 0);
        assert_eq!(LuaValidator::levenshtein("put", "get"), 2);
        assert_eq!(LuaValidator::levenshtein("semantic_search", "semanticsearch"), 1);
    }

    #[test]
    fn test_available_functions() {
        let validator = LuaValidator::new();
        let funcs = validator.get_available_functions();
        assert!(!funcs.is_empty());
        assert!(funcs.iter().any(|f| f.name == "put"));
        assert!(funcs.iter().any(|f| f.name == "semantic_search"));
    }
}
