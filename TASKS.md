# Tasks to Transform liath-rs into a Proper Cargo Library

## 1. Restructure Cargo.toml
- [x] Split into library and binary sections
- [x] Rename package from "whitematter" to "liath"
- [x] Add proper metadata for publishing

## 2. Create Library Interface
- [x] Create src/lib.rs with public API
- [x] Expose core modules as public interfaces
- [x] Create an EmbeddedLiath equivalent struct
- [x] Define clear public API boundaries

## 3. Refactor Main Entry Point
- [x] Move main.rs functionality to src/bin/liath.rs
- [x] Simplify binary to use library functions

## 4. Improve Module Structure
- [ ] Ensure all public types are properly exported
- [ ] Add documentation to public interfaces
- [ ] Organize modules logically

## 5. Add Configuration Support
- [ ] Create config module for library configuration
- [ ] Support configuration via TOML files
- [ ] Provide programmatic configuration options

## 6. Enhance Error Handling
- [ ] Create custom error types for the library
- [ ] Ensure errors are properly exposed in public API

## 7. Add Documentation
- [x] Add module-level documentation
- [x] Add examples in documentation
- [x] Create README.md updates

## 8. Testing
- [ ] Add unit tests for library functionality
- [ ] Add integration tests
- [ ] Ensure library can be used as dependency

## 9. Build and Verification
- [ ] Verify library builds correctly (blocked by system dependencies)
- [ ] Verify binary installs correctly
- [ ] Test library usage in example project

## 10. Additional Improvements
- [x] Document system dependencies
- [ ] Implement CLI functionality
- [ ] Implement server functionality
- [ ] Add proper error handling to EmbeddedLiath
- [ ] Add namespace support to EmbeddedLiath