use async_graphql::Context;
use sqlx::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct Article {
    id: i32,
    title: String,
    content: String,
}

#[async_graphql::Object]
impl Article {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn title(&self) -> String {
        self.title.clone()
    }

    async fn content(&self) -> String {
        self.content.clone()
    }

    async fn author(&self, ctx: &Context<'_>) -> Author {
        // SQLx wraps the pool in a Arc so clone is not expensive
        let pool = ctx.data::<crate::GraphQLContext>().pool.clone();

        let row = sqlx::query_as::<_, Author>(
            r"
              SELECT s.* FROM article a inner join author s on a.author_id = s.id where a.id = $1
             ",
        )
        .bind(self.id)
        .fetch_one(&pool)
        .await
        .unwrap();

        row
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Author {
    id: i32,
    name: String,
    password: String,
}

#[async_graphql::Object]
impl Author {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn password(&self) -> String {
        self.password.clone()
    }

    async fn articles(&self, context: &Context<'_>) -> Vec<Article> {
        let pool = context.data::<crate::GraphQLContext>().pool.clone();

        let rows = sqlx::query_as::<_, Article>("SELECT * FROM article where author_id = $1")
            .bind(self.id)
            .fetch_all(&pool)
            .await
            .unwrap();
        rows
    }
}

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn articles(&self, context: &Context<'_>) -> Vec<Article> {
        // SQLx wraps the pool in a Arc so clone is not expensive
        let pool = context.data::<crate::GraphQLContext>().pool.clone();

        let rows = sqlx::query_as::<_, Article>("SELECT * FROM article")
            .fetch_all(&pool)
            .await
            .unwrap();

        rows
    }
}

pub struct MutationRoot;

#[async_graphql::InputObject]
struct CreateUserArgs {
    title: String,
    content: String,
    author_id: i32,
}

#[async_graphql::Object]
impl MutationRoot {
    async fn createArticle(&self, context: &Context<'_>, data: CreateUserArgs) -> Article {
        let pool = context.data::<crate::GraphQLContext>().pool.clone();

        let row = sqlx::query_as::<_, Article>(
            "INSERT INTO article(title, content, author_id) VALUES ($1,$2,$3) RETURNING *",
        )
        .bind(data.title)
        .bind(data.content)
        .bind(data.author_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        row
    }
}
