use fjall::{Config, Keyspace, PartitionHandle, PartitionCreateOptions};
use std::path::Path;
use anyhow::{Result, Context};

pub struct FjallWrapper {
    keyspace: Keyspace,
    partition: PartitionHandle,
}

impl FjallWrapper {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let keyspace = Config::new(path)
            .open()
            .context("Failed to open Fjall keyspace")?;
        
        let partition = keyspace
            .open_partition("default", PartitionCreateOptions::default())
            .context("Failed to open default partition")?;
        
        Ok(Self { 
            keyspace,
            partition,
        })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.partition.insert(key, value)
            .context("Failed to put value in DB")?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let res = self.partition
            .get(key)
            .context("Failed to get value from DB")?;
        Ok(res.map(|slice| slice.to_vec()))
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.partition.remove(key)
            .context("Failed to delete value from DB")?;
        Ok(())
    }

    // Method to perform batch operations
    pub fn batch_put(&self, items: Vec<(&[u8], &[u8])>) -> Result<()> {
        let mut batch = self.keyspace.batch();
        for (key, value) in items {
            batch.insert(&self.partition, key, value);
        }
        batch.commit()
            .context("Failed to commit batch")?;
        Ok(())
    }
}
