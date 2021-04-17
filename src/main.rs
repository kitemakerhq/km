#![allow(proc_macro_derive_resolution_fallback)] // We need this because graphql_client hasn't been updated towards latest Rust spec.

use structopt::StructOpt;

use anyhow::*;
use graphql_client::*;


/// Command line tool for Kitemaker
#[derive(StructOpt)]
#[structopt(about = "Command line tool for Kitemaker")]
struct Cli {
    #[structopt(short, long, env = "KM_TOKEN")]
    token: String,

    #[structopt(subcommand)]
    cmd: Commands
}

#[derive(StructOpt)]
enum Commands {
    /// List organization name and ID
    Organization,
    /// List all work items in a space
    Spaces,
}

//
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug"
)]
struct OrgQuery;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug"
)]
struct SpaceQuery;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::from_args(); 

    let client = reqwest::Client::new();

    match args.cmd {
        Commands::Organization => {
            let q = OrgQuery::build_query( org_query::Variables {});

            let res = client
                .post("https://toil.kitemaker.co/developers/graphql")
                .bearer_auth(args.token)
                .json(&q)
                .send().await?;


            res.error_for_status_ref()?;

            let response_body: Response<org_query::ResponseData> = res.json().await?;
            

            let response_data: org_query::ResponseData = response_body.data.expect("missing response data");
            println!("Organization Name: {:?}", response_data.organization.name);
            println!("Organization ID: {:?}", response_data.organization.id);
        }


        Commands::Spaces => {
            let q = SpaceQuery::build_query( space_query::Variables {});

            let res = client
                .post("https://toil.kitemaker.co/developers/graphql")
                .bearer_auth(args.token)
                .json(&q)
                .send().await?;


            res.error_for_status_ref()?;

            let response_body: Response<space_query::ResponseData> = res.json().await?;
            
            let response_data: space_query::ResponseData = response_body.data.expect("missing response data");
            println!("Organization Name: {:?}", response_data.organization.name);
            println!("Organization ID: {:?}", response_data.organization.id);
            println!("Number of spaces: {:?}", response_data.organization.spaces.len());

            for space in response_data.organization.spaces.iter() {
                println!("Space Name: {:?}", space.name);
                println!("Space Id: {:?}", space.id);
            }
        }
    }

    Ok(())
}
