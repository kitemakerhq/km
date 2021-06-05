#![allow(proc_macro_derive_resolution_fallback)] // We need this because graphql_client hasn't been updated towards latest Rust spec.

use structopt::StructOpt;

use anyhow::*;
use graphql_client::*;

use colored::*;

use colors_transform::Color;
use colors_transform::Rgb;


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
    /// List organization name and all spaces
    Organization,
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
        /// Optional space key
        space: Option<String>,
        /// List all (also backlog and done)
        #[structopt(short, long)]
        all: bool,
        /// List only backlog
        #[structopt(short, long)]
        backlog: bool,
    },

    /// Create a new work item
    Create {
        /// Key for the target space
        space: String,
        /// Title of the new work item
        title: String,
        /// Description as markdown formatted text
        description: Option<String>,
    },

    /// View a work item
    View {
        /// The number with space key (e.g., ABC-123) for the work item
        number: String
    }
}

// GraphQL queries
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug"
)]
struct ItemQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug"
)]
struct CreateWorkItem;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::from_args(); 

    let client = reqwest::Client::new();

    match args.cmd {
        Commands::Organization => {
            let q = SpaceQuery::build_query( space_query::Variables {});

            let res = client
                .post("https://toil.kitemaker.co/developers/graphql")
                .bearer_auth(args.token)
                .json(&q)
                .send().await?;


            res.error_for_status_ref()?;

            let response_body: Response<space_query::ResponseData> = res.json().await?;
    
            let response_data: space_query::ResponseData = response_body.data.expect("missing response data");
            
            println!("{:}", response_data.organization.name.bold());

            println!("{:<15}{:}", "Key".bold().underline(),"Space name".bold().underline());
            for space in response_data.organization.spaces.iter() {
                println!("{:<15}{:}",space.key.yellow(), space.name.bold());
            }
        }


        Commands::Item(arg) => {
            
            match arg.cmd {
                Item::List{space, all, backlog} => {

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
                            println!("{:<30}{:<20}{:<40}", "Status".bold().underline(),"Key".bold().underline(),"Title".bold().underline());

                            let mut has_more = true;
                            let mut cursor: Option<String> = None;

                            while has_more  {
                                let q = ItemsQuery::build_query( items_query::Variables {space_id: spc.id.to_string(), cursor: cursor});

                                let res = client
                                    .post("https://toil.kitemaker.co/developers/graphql")
                                    .bearer_auth(args.token.to_string())
                                    .json(&q)
                                    .send().await?;
                    
                    
                                res.error_for_status_ref()?;
                    
                                let response_body: Response<items_query::ResponseData> = res.json().await?;
                                
                                let response_data: items_query::ResponseData = response_body.data.expect("missing response data");

                                has_more = response_data.work_items.has_more;
                                cursor = Some(response_data.work_items.cursor.to_string());

                                for item in response_data.work_items.work_items {

                                    if (all && item.status.type_ != items_query::StatusType::ARCHIVED) ||
                                    (backlog && item.status.type_ == items_query::StatusType::BACKLOG) ||
                                    (!backlog && !all && (item.status.type_ == items_query::StatusType::TODO || item.status.type_ == items_query::StatusType::IN_PROGRESS)) {
                                    
                                        let mut labels = format!("");
                                        for label in item.labels {

                                            let rgb = Rgb::from_hex_str(&label.color).unwrap();
                                            labels = format!("{:} {:}{:}", labels, "#".truecolor( rgb.get_red() as u8, rgb.get_green() as u8, rgb.get_blue() as u8 ), label.name);
                                        }
                                        
                                        println!("{:<30}{:<20}{:} {:}", item.status.name,spc.key.to_string() + "-" + &item.number.to_string(),item.title, labels.italic());
                                    }
                                }
                            }
                        }
                    }
                }
                Item::Create{space, title, description} => {
                    let q = SpaceQuery::build_query( space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send().await?;
        
        
                    res.error_for_status_ref()?;
        
                    let response_body: Response<space_query::ResponseData> = res.json().await?;
                    
                    let response_data: space_query::ResponseData = response_body.data.expect("missing response data");

                    let spc = response_data.organization.spaces.iter().find( |&s| s.key == space);

                    match spc {
                        None => { println!("Could not find space {:}", space); }
                        Some(s) => { 

                            
                            // Find the default status
                            let default_status = s.statuses.iter().find( | &st | st.default == true).expect("missing default status");

                            let q = CreateWorkItem::build_query( create_work_item::Variables {
                                status_id: default_status.id.to_string(),
                                title: title,
                                description: description,
                            });

                            let res = client
                                .post("https://toil.kitemaker.co/developers/graphql")
                                .bearer_auth(args.token.to_string())
                                .json(&q)
                                .send().await?;

                            res.error_for_status_ref()?;
    
                            let response_body: Response<create_work_item::ResponseData> = res.json().await?;
                            
                            let response_data: create_work_item::ResponseData = response_body.data.expect("missing response data");

                            let work_item_number = format!("{:}-{:}",s.key, response_data.create_work_item.work_item.number);
                            println!("Work item {:} created", work_item_number.bold());
                        }
                    }
                }
                Item::View{number} => {
                    let mut parts = number.split("-");
                    let space = parts.next().expect("Missing space key");
                    let number = parts.next().expect("Missing work item number");

                    // First find the correct space and id
                    let q = SpaceQuery::build_query( space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send().await?;
        
        
                    res.error_for_status_ref()?;
        
                    let response_body: Response<space_query::ResponseData> = res.json().await?;
                    
                    let response_data: space_query::ResponseData = response_body.data.expect("missing response data");
        
                    let spc = response_data.organization.spaces.iter().find( |&s| s.key == space);

                    match spc {
                        None => { println!("Could not find space {:}", space); }
                        Some(s) => { 
                            // Find the work item
                            let mut has_more = true;
                            let mut cursor: Option<String> = None;

                            while has_more  {
                                let q = ItemsQuery::build_query( items_query::Variables {space_id: s.id.to_string(), cursor: cursor});

                                let res = client
                                    .post("https://toil.kitemaker.co/developers/graphql")
                                    .bearer_auth(args.token.to_string())
                                    .json(&q)
                                    .send().await?;
                    
                    
                                res.error_for_status_ref()?;
                    
                                let response_body: Response<items_query::ResponseData> = res.json().await?;
                                
                                let response_data: items_query::ResponseData = response_body.data.expect("missing response data");

                                has_more = response_data.work_items.has_more;
                                cursor = Some(response_data.work_items.cursor.to_string());

                                let item = response_data.work_items.work_items.iter().find( |&w| w.number == number);

                                match item {
                                    None => {

                                    },
                                    Some(i) => {

                                        let q = ItemQuery::build_query( item_query::Variables {item_id: i.id.to_string()});

                                        let res = client
                                            .post("https://toil.kitemaker.co/developers/graphql")
                                            .bearer_auth(args.token.to_string())
                                            .json(&q)
                                            .send().await?;
                            
                                        res.error_for_status_ref()?;

                                        let response_body: Response<item_query::ResponseData> = res.json().await?;
                                        
                                        let response_data: item_query::ResponseData = response_body.data.expect("missing response data");

                                        let item = response_data.work_item;

                                        println!("{}-{}: {}", s.key.bold(), item.number.bold(), item.title.bold());

                                        if item.labels.len() > 0 {
                                            let mut labels = format!("");
                                            for label in item.labels {

                                                let rgb = Rgb::from_hex_str(&label.color).unwrap();
                                                labels = format!("{:} {:}{:}", labels, "#".truecolor( rgb.get_red() as u8, rgb.get_green() as u8, rgb.get_blue() as u8 ), label.name);
                                            }
                                            println!("{} {}", "Labels:".bold(), labels);
                                        }
                                        println!("\n{}\n{}","Description:".bold(), item.description);
                                        
                                        // We're done, so skip searching for more items
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}