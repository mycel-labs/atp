use crate::traits::model::Model;

/// Repository trait providing high-level database operations for models
pub trait Repository<T: Model> {
    type Error;

    /// Insert or update a model instance
    fn save(&self, model: &T) -> Result<T, Self::Error>;

    /// Get a model by its primary key
    fn find_by_id(&self, id: &T::PrimaryKey) -> Result<Option<T>, Self::Error>;

    /// Get all models
    fn find_all(&self) -> Result<Vec<T>, Self::Error>;

    /// Delete a model by its primary key
    fn delete(&self, id: &T::PrimaryKey) -> Result<bool, Self::Error>;

    /// Check if a model exists by its primary key
    fn exists(&self, id: &T::PrimaryKey) -> Result<bool, Self::Error>;
}
