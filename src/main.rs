use structopt::StructOpt;

use graphql_client::{GraphQLQuery, Response};

use std::error::Error;

use colored::*;

use colors_transform::Color;
use colors_transform::Rgb;

use std::collections::HashMap;

// GraphQL queries
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug,PartialEq"
)]
struct SpaceQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug,PartialEq"
)]
struct ItemsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug,PartialEq"
)]
struct ItemQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/kitemaker.graphql",
    query_path = "src/queries.graphql",
    response_derives = "Debug,PartialEq"
)]
struct CreateWorkItem;

/// Command line tool for Kitemaker
#[derive(StructOpt)]
#[structopt(about = "Command line tool for Kitemaker")]
struct Cli {
    #[structopt(short, long, env = "KM_TOKEN")]
    token: String,

    #[structopt(subcommand)]
    cmd: Commands,
}

#[derive(StructOpt)]
enum Commands {
    /// List organization name, members, and all spaces
    Organization,
    /// Work items subcommands
    Item(SubCommands),
}

#[derive(StructOpt, Debug)]
struct SubCommands {
    #[structopt(subcommand)]
    cmd: Item,
}

#[derive(StructOpt, Debug)]
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
        number: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    let client = reqwest::Client::new();

    let color_mapping: HashMap<&str, &str> = [
        ("gray", "#8D8D8D"),
        ("mauve", "#8E8C99"),
        ("slate", "#8B8D98"),
        ("sage", "#B8BCBA"),
        ("olive", "#898E87"),
        ("sand", "#8D8D86"),
        ("tomato", "#E54D2E"),
        ("red", "#E5484D"),
        ("ruby", "#E54666"),
        ("crimson", "#E93D82"),
        ("pink", "#D6409F"),
        ("plum", "#AB4ABA"),
        ("purple", "#8E4EC6"),
        ("violet", "#6E56CF"),
        ("iris", "#5B5BD6"),
        ("indigo", "#3E63DD"),
        ("blue", "#0090FF"),
        ("cyan", "#00A2C7"),
        ("teal", "#12A594"),
        ("jade", "#29A383"),
        ("green", "#30A46C"),
        ("grass", "#46A758"),
        ("bronze", "#A18072"),
        ("gold", "#978365"),
        ("brown", "#AD7F58"),
        ("orange", "#F76B15"),
        ("amber", "#FFC53D"),
        ("yellow", "#FFE629"),
        ("lime", "#BDEE63"),
        ("mint", "#86EAD4"),
        ("sky", "#7CE2FE"),
    ]
    .into();

    match args.cmd {
        Commands::Organization => {
            let q = SpaceQuery::build_query(space_query::Variables {});

            let res = client
                .post("https://toil.kitemaker.co/developers/graphql")
                .bearer_auth(args.token)
                .json(&q)
                .send()
                .await?;

            res.error_for_status_ref()?;

            let response_json: Response<space_query::ResponseData> = res.json().await?;
            let response_data: space_query::SpaceQueryOrganization =
                response_json.data.unwrap().organization;

            println!(
                "{:} {:}\n",
                "Organization:".bold().underline().yellow(),
                response_data.name.bold().underline()
            );

            println!(
                "{:<15}{:<25}{:}",
                "Username".bold().underline(),
                "Name".bold().underline(),
                "Guest".bold().underline()
            );

            for user in response_data.users.iter() {
                println!(
                    "{:<15}{:<25}{:}",
                    user.username.yellow(),
                    if user.name.is_some() {
                        user.name.as_ref().unwrap()
                    } else {
                        ""
                    },
                    if user.guest { "yes" } else { "" }
                );
            }
            println!(
                "\n{:<15}{:}",
                "Key".bold().underline(),
                "Space name".bold().underline()
            );
            for space in response_data.spaces.iter() {
                println!("{:<15}{:}", space.key.yellow(), space.name.bold());
            }
        }

        Commands::Item(arg) => {
            match arg.cmd {
                Item::List {
                    space,
                    all,
                    backlog,
                } => {
                    let q = SpaceQuery::build_query(space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send()
                        .await?;

                    res.error_for_status_ref()?;

                    let response_json: Response<space_query::ResponseData> = res.json().await?;
                    let response_data = response_json.data.unwrap();

                    for spc in response_data.organization.spaces.iter() {
                        let mut print_items = false;

                        match space {
                            None => {
                                println!("\n\n{:} {:}", "Space:".bold(), spc.name.bold());
                                print_items = true;
                            }
                            Some(ref x) => {
                                if x == &spc.key.to_string() {
                                    print_items = true;
                                }
                            }
                        }

                        if print_items {
                            println!(
                                "{:<30}{:<20}{:<40}",
                                "Status".bold().underline(),
                                "Key".bold().underline(),
                                "Title".bold().underline()
                            );

                            let mut has_more = true;
                            let mut cursor: Option<String> = None;

                            while has_more {
                                let q = ItemsQuery::build_query(items_query::Variables {
                                    space_id: spc.id.to_string(),
                                    cursor: cursor,
                                });

                                let res = client
                                    .post("https://toil.kitemaker.co/developers/graphql")
                                    .bearer_auth(args.token.to_string())
                                    .json(&q)
                                    .send()
                                    .await?;

                                res.error_for_status_ref()?;

                                let response_json: Response<items_query::ResponseData> =
                                    res.json().await?;
                                let response_data = response_json.data.unwrap();

                                has_more = response_data.work_items.has_more;
                                cursor = Some(response_data.work_items.cursor.to_string());

                                for item in response_data.work_items.work_items {
                                    if (all
                                        && item.status.type_ != items_query::StatusType::ARCHIVED)
                                        || (backlog
                                            && item.status.type_
                                                == items_query::StatusType::BACKLOG)
                                        || (!backlog
                                            && !all
                                            && (item.status.type_ == items_query::StatusType::TODO
                                                || item.status.type_
                                                    == items_query::StatusType::IN_PROGRESS))
                                    {
                                        let mut labels = format!("");
                                        for label in item.labels {
                                            let hex_color = if label.color.starts_with('#') {
                                                label.color.clone()
                                            } else {
                                                color_mapping
                                                    .get(&label.color.as_str())
                                                    .unwrap_or(&"#FFFFFF")
                                                    .to_string()
                                            };
                                            let rgb = Rgb::from_hex_str(&hex_color).unwrap();
                                            labels = format!(
                                                "{:} {:}",
                                                labels,
                                                label.name.truecolor(
                                                    rgb.get_red() as u8,
                                                    rgb.get_green() as u8,
                                                    rgb.get_blue() as u8
                                                )
                                            );
                                        }

                                        println!(
                                            "{:<30}{:<20}{:} {:}",
                                            item.status.name,
                                            spc.key.to_string() + "-" + &item.number.to_string(),
                                            item.title,
                                            labels.italic()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Item::Create {
                    space,
                    title,
                    description,
                } => {
                    let q = SpaceQuery::build_query(space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send()
                        .await?;

                    res.error_for_status_ref()?;

                    let response_json: Response<space_query::ResponseData> = res.json().await?;
                    let response_data = response_json.data.unwrap();

                    let spc = response_data
                        .organization
                        .spaces
                        .iter()
                        .find(|&s| s.key == space);

                    match spc {
                        None => {
                            println!("Could not find space {:}", space);
                        }
                        Some(s) => {
                            // Find the default status
                            let default_status = s
                                .statuses
                                .iter()
                                .find(|&st| {
                                    st.default == true
                                        && (st.type_ == space_query::StatusType::BACKLOG
                                            || st.type_ == space_query::StatusType::TODO)
                                })
                                .expect("missing default open status");

                            let q = CreateWorkItem::build_query(create_work_item::Variables {
                                status_id: default_status.id.to_string(),
                                title: title,
                                description: description,
                            });

                            let res = client
                                .post("https://toil.kitemaker.co/developers/graphql")
                                .bearer_auth(args.token.to_string())
                                .json(&q)
                                .send()
                                .await?;

                            res.error_for_status_ref()?;

                            let response_json: Response<create_work_item::ResponseData> =
                                res.json().await?;
                            let response_data = response_json.data.unwrap();

                            let work_item_number = format!(
                                "{:}-{:}",
                                s.key, response_data.create_work_item.work_item.number
                            );
                            println!("Work item {:} created", work_item_number.bold());
                        }
                    }
                }
                Item::View { number } => {
                    let mut parts = number.split("-");
                    let space = parts.next().expect("Missing space key");
                    let number = parts.next().expect("Missing work item number");

                    // First find the correct space and id
                    let q = SpaceQuery::build_query(space_query::Variables {});

                    let res = client
                        .post("https://toil.kitemaker.co/developers/graphql")
                        .bearer_auth(args.token.to_string())
                        .json(&q)
                        .send()
                        .await?;

                    res.error_for_status_ref()?;

                    let response_json: Response<space_query::ResponseData> = res.json().await?;
                    let response_data = response_json.data.unwrap();

                    let spc = response_data
                        .organization
                        .spaces
                        .iter()
                        .find(|&s| s.key == space);

                    match spc {
                        None => {
                            println!("Could not find space {:}", space);
                        }
                        Some(s) => {
                            // Find the work item
                            let mut has_more = true;
                            let mut cursor: Option<String> = None;

                            while has_more {
                                let q = ItemsQuery::build_query(items_query::Variables {
                                    space_id: s.id.to_string(),
                                    cursor: cursor,
                                });

                                let res = client
                                    .post("https://toil.kitemaker.co/developers/graphql")
                                    .bearer_auth(args.token.to_string())
                                    .json(&q)
                                    .send()
                                    .await?;

                                res.error_for_status_ref()?;

                                let response_json: Response<items_query::ResponseData> =
                                    res.json().await?;
                                let response_data = response_json.data.unwrap();

                                has_more = response_data.work_items.has_more;
                                cursor = Some(response_data.work_items.cursor.to_string());

                                let item = response_data
                                    .work_items
                                    .work_items
                                    .iter()
                                    .find(|&w| w.number == number);

                                match item {
                                    None => {}
                                    Some(i) => {
                                        let q = ItemQuery::build_query(item_query::Variables {
                                            item_id: i.id.to_string(),
                                        });

                                        let res = client
                                            .post("https://toil.kitemaker.co/developers/graphql")
                                            .bearer_auth(args.token.to_string())
                                            .json(&q)
                                            .send()
                                            .await?;

                                        res.error_for_status_ref()?;

                                        let response_json: Response<item_query::ResponseData> =
                                            res.json().await?;
                                        let response_data = response_json.data.unwrap();

                                        let item = response_data.work_item;

                                        println!(
                                            "{}-{}: {}",
                                            s.key.bold(),
                                            item.number.bold(),
                                            item.title.bold()
                                        );
                                        println!("Status: {}", item.status.name.bold());

                                        if item.labels.len() > 0 {
                                            let mut labels = format!("");
                                            for label in item.labels {
                                                let hex_color = if label.color.starts_with('#') {
                                                    label.color.clone()
                                                } else {
                                                    color_mapping
                                                        .get(&label.color.as_str())
                                                        .unwrap_or(&"#FFFFFF")
                                                        .to_string()
                                                };
                                                let rgb = Rgb::from_hex_str(&hex_color).unwrap();
                                                labels = format!(
                                                    "{:} {:}",
                                                    labels,
                                                    label.name.truecolor(
                                                        rgb.get_red() as u8,
                                                        rgb.get_green() as u8,
                                                        rgb.get_blue() as u8
                                                    ),
                                                );
                                            }
                                            println!("{}{}", "Labels:", labels);
                                        }
                                        println!("\n{}\n", "Description:".bold());
                                        termimad::print_text(item.description.as_str());

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
