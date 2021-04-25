#![allow(proc_macro_derive_resolution_fallback)] // We need this because graphql_client hasn't been updated towards latest Rust spec.

use structopt::StructOpt;

use anyhow::*;
use graphql_client::*;

use colored::*;

use colors_transform::*;
use colors_transform::Color;


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
    /// List all spaces in the organization
    Spaces,
    /// Work items subcommands
    Item(SubCommands),
}

#[derive(StructOpt, Debug)]
struct SubCommands {
    #[structopt(subcommand)]
    cmd: Item,
}


#[derive(StructOpt,Debug)]
enum Item {
    /// List all work items
    List {
        space: Option<String>,
    },

    /// Create a new work item
    Create {
        space: String,
        title: String
    },
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


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug"
)]
struct ItemsQuery;

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
            
            println!("{:} {:}", response_data.organization.name.bold(), response_data.organization.id.italic());
        }


        Commands::Spaces => {
            let q = SpaceQuery::build_query( space_query::Variables {});

            let res = client
                .post("https://toil.kitemaker.co/developers/graphql")
                .bearer_auth(args.token.to_string())
                .json(&q)
                .send().await?;


            res.error_for_status_ref()?;

            let response_body: Response<space_query::ResponseData> = res.json().await?;
            
            let response_data: space_query::ResponseData = response_body.data.expect("missing response data");

            println!("{}\t\t{}", "Key".bold().underline(),"Name".bold().underline());
            for space in response_data.organization.spaces.iter() {
                println!("{:}\t\t{:}",space.key.yellow(), space.name.bold());
            }
        }

        Commands::Item(arg) => {
            
            match arg.cmd {
                Item::List{space} => {

                    let q = SpaceQuery::build_query( space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send().await?;
        
        
                    res.error_for_status_ref()?;
        
                    let response_body: Response<space_query::ResponseData> = res.json().await?;
                    
                    let response_data: space_query::ResponseData = response_body.data.expect("missing response data");
        
                    for spc in response_data.organization.spaces.iter() {

                        let mut print_items = false;

                        match space {
                            None => { 
                                println!("\n\n{:} {:}","Space:".bold(), spc.name.bold());
                                print_items = true;
                            }
                            Some( ref x) => {
                                if x == &spc.key.to_string() {
                                    print_items = true; 
                                }
                            }
                        }
                        
                        if print_items {
                            println!("{:<20}{:<20}{:<40}", "Status".bold().underline(),"Key".bold().underline(),"Title".bold().underline());


                            let q = ItemsQuery::build_query( items_query::Variables {space_id: spc.id.to_string()});

                            let res = client
                                .post("https://toil.kitemaker.co/developers/graphql")
                                .bearer_auth(args.token.to_string())
                                .json(&q)
                                .send().await?;
                
                
                            res.error_for_status_ref()?;
                
                            let response_body: Response<items_query::ResponseData> = res.json().await?;
                            
                            let response_data: items_query::ResponseData = response_body.data.expect("missing response data");

                            for item in response_data.work_items.work_items {
                                
                                let mut labels = format!("");
                                for label in item.labels {

                                    let rgb = Rgb::from_hex_str(&label.color).unwrap();
                                    labels = format!("{:} {:}{:}", labels, "#".truecolor( rgb.get_red() as u8, rgb.get_green() as u8, rgb.get_blue() as u8 ), label.name);
                                }
                                
                                println!("{:<20}{:<20}{:} {:}", item.status.name,spc.key.to_string() + "-" + &item.number.to_string(),item.title, labels.italic());
                            }
                        }
                    }
                }
                Item::Create{space, title} => {
                    println!("Let's create the stuff in space{:} and name it {:}!!!", space, title);
                }
            }
        }
    }
    Ok(())
}