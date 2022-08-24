use log::info;
use rand::Rng;
use rspc::RouterBuilder;

use super::Shared;
use rspc::Type;
use serde::{Deserialize, Serialize};

#[derive(Type, Deserialize, Debug, Serialize)]
pub struct UpdateSpaceIndexesArgs {
    pub spaces: Vec<prisma::spaces::Data>,
}

pub fn mount() -> RouterBuilder<Shared> {
    RouterBuilder::<Shared>::new()
        .query("get", move |ctx, _: ()| async move {
            let client = ctx.client.clone();
            let mut spaces = client.spaces().find_many(vec![]).exec().await?;
            spaces.sort_by(|a, b| a.index.cmp(&b.index));
            Ok(spaces)
        })
        .mutation("create", move |ctx, _: ()| async move {
            let client = ctx.client.clone();

            let colors = vec![
                "var(--colors-blue)",
                "var(--colors-red)",
                "var(--colors-green)",
                "var(--colors-yellow)",
                "var(--colors-pink)",
                "var(--colors-teal)",
                "var(--colors-mauve)",
            ];

            let color = colors[rand::thread_rng().gen_range(0, colors.len())];

            let spaces = client.spaces().find_many(vec![]).exec().await?.len();
            info!("Creating space with id: {}", spaces);
            let space = client
                .spaces()
                .create(
                    "Space ".to_owned() + &spaces.to_string(),
                    String::new(),
                    String::from("Document16Filled"),
                    String::from(color),
                    vec![prisma::spaces::index::set(spaces.try_into().unwrap())],
                )
                .exec()
                .await?;

            Ok(space)
        })
        .mutation(
            "updateSpaceIndexes",
            move |ctx, args: UpdateSpaceIndexesArgs| async move {
                let client = ctx.client.clone();
                info!("{:?}", args);
                let mut edited_spaces = Vec::new();
                for (index, space) in args.spaces.iter().enumerate() {
                    // create spaces if they do not exist
                    let inserted_space = client
                        .spaces()
                        .upsert(
                            prisma::spaces::id::equals(space.id.clone()),
                            (
                                "Space ".to_owned() + &(space.id + 1).to_string(),
                                String::new(),
                                String::from("Document16Filled"),
                                String::from("#ffffff"),
                                vec![prisma::spaces::index::set(index.try_into().unwrap())],
                            ),
                            vec![prisma::spaces::index::set(index.try_into().unwrap())],
                        )
                        .exec();

                    edited_spaces.push(inserted_space);
                }

                futures::future::join_all(edited_spaces).await;

                Ok(())
            },
        )
}
