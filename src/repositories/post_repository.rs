use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::schema::posts::dsl::*;
use crate::models::post::{Post, CreatePostPayload, UpdatePostPayload};
use crate::errors::AppError;
use crate::models::category::Category;
use diesel::BelongingToDsl;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct PostRepository {
    pool: DbPool,
}

impl PostRepository {
    pub fn new(pool: DbPool) -> Self {
        PostRepository { pool }
    }

    pub async fn create_post(&self, new_post: CreatePostPayload, current_user_id: i32) -> Result<Post, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            let post_data = (
                title.eq(&new_post.title),
                content.eq(&new_post.content),
                category_id.eq(new_post.category_id),
                crate::schema::posts::dsl::user_id.eq(current_user_id),
            );
            Ok(diesel::insert_into(posts)
                .values(post_data)
                .returning(Post::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn get_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            let total = posts.into_boxed().count().get_result::<i64>(&mut conn)?;
            let result = posts.into_boxed().limit(limit).offset(offset).select(Post::as_select()).load(&mut conn)?;
            Ok((result, total))
        })
        .await?
    }

    pub async fn get_post_by_id(&self, post_id: i32) -> Result<Post, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(posts.find(post_id).select(Post::as_select()).first(&mut conn)?)
        })
        .await?
    }

    pub async fn get_posts_by_category(&self, slug_path: String) -> Result<Vec<Post>, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            use crate::schema::categories::dsl as categories_dsl;

            let category = categories_dsl::categories
                .filter(categories_dsl::slug.eq(slug_path))
                .select(Category::as_select())
                .first(&mut conn)?;

            Ok(Post::belonging_to(&category)
                .select(Post::as_select())
                .load(&mut conn)?)
        })
        .await?
    }

    pub async fn update_post(&self, post_id: i32, update_payload: UpdatePostPayload) -> Result<Post, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::update(posts.find(post_id))
                .set(&update_payload)
                .returning(Post::as_returning())
                .get_result(&mut conn)?)
        })
        .await?
    }

    pub async fn delete_post(&self, post_id: i32) -> Result<usize, AppError> {
        let mut conn = self.pool.get().expect("Failed to get a connection");
        tokio::task::spawn_blocking(move || {
            Ok(diesel::delete(posts.find(post_id)).execute(&mut conn)?)
        })
        .await?
    }
}
