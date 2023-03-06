use std::error::Error;

use bson::Document;
use mongodb::options::{
    ClientOptions,
    DeleteOptions,
    FindOneAndDeleteOptions,
    FindOptions,
    InsertOneOptions,
    ResolverConfig,
    UpdateModifications,
    UpdateOptions, FindOneOptions,
};
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::{Client, Collection, Cursor};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct Database {
    client: Client,
    database: String,
}

impl Database {
    pub async fn new(connection: &str, database: &str) -> Result<Database, Box<dyn Error>> {
        let options =
            ClientOptions::parse_with_resolver_config(&connection, ResolverConfig::cloudflare())
                .await?;
        let client = Client::with_options(options)?;

        Ok(Database {
            client,
            database: database.to_string(),
        })
    }

    fn open<T>(&self, collection: &str) -> Collection<T> {
        self.client.database(&self.database).collection(collection)
    }

    pub async fn update<T>(
        &self,
        collection: &str,
        query: Document,
        update_modifications: UpdateModifications,
        options: Option<UpdateOptions>,
    ) -> Result<UpdateResult, Box<dyn Error + Send + Sync>> {
        let result = self
            .open::<T>(collection)
            .update_one(query, update_modifications, options)
            .await?;

        Ok(result)
    }

    pub async fn find<T>(
        &self,
        collection: &str,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<Cursor<T>, Box<dyn Error + Send + Sync>> {
        let cursor = self.open(collection).find(filter, options).await?;

        Ok(cursor)
    }

    pub async fn find_one<T: DeserializeOwned + Send + Sync + Unpin>(
        &self,
        collection: &str,
        filter: Document,
        options: Option<FindOneOptions>
    ) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let doc = self.open::<T>(collection).find_one(filter, options).await?;

        Ok(doc)
    }

    pub async fn find_and_delete<T: DeserializeOwned>(
        &self,
        collection: &str,
        filter: Document,
        options: Option<FindOneAndDeleteOptions>,
    ) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let result = self
            .open(collection)
            .find_one_and_delete(filter, options)
            .await?;

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn insert<T: Serialize>(
        &self,
        collection: &str,
        doc: T,
        options: Option<InsertOneOptions>,
    ) -> Result<InsertOneResult, Box<dyn Error + Send + Sync>> {
        let result = self.open(collection).insert_one(doc, options).await?;

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn delete(
        &self,
        collection: &str,
        query: Document,
        options: Option<DeleteOptions>,
    ) -> Result<DeleteResult, Box<dyn Error + Send + Sync>> {
        let result = self
            .open::<Document>(collection)
            .delete_one(query, options)
            .await?;

        Ok(result)
    }
}
