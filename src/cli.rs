use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "CLI for development, upload game builds to Raccreative Games", long_about = Some("

   _____ _                   _                 
  / ____| |                 | |                
 | |    | | __ ___      ____| |_ __ ___  _ __  
 | |    | |/ _` \\ \\ /\\ / / _` | '__/ _ \\| '_ \\ 
 | |____| | (_| |\\ V  V / (_| | | | (_) | |_) |
  \\_____|_|\\__,_| \\_/\\_/ \\__,_|_|  \\___/| .__/ 
                                        | |    
                                        |_|    

 Clawdrop is a command line tool for indie game developers for Raccreative Games."))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Development environment diagnostics")]
    Doctor,
    #[command(about = "Authorization to use API Key from Raccreative [Opens URL]")]
    Auth {
        #[arg(long, help = "Ignores previous API Key")]
        force: bool,

        #[arg(long, help = "Prevents automatic browser opening")]
        headless: bool,

        #[arg(long, short = 'k', help = "Authorizes with given API Key")]
        key: Option<String>,
    },
    #[command(about = "Remove API Key and log out")]
    Logout,
    #[command(about = "Shows a list with the games you have permissions to upload builds")]
    List,
    #[command(about = "Updates Clawdrop to the latest version")]
    Upgrade,
    #[command(
        about = "Sets a game via id or url slug to be the main target of clawdrop. clawdrop set <ID/url-slug>"
    )]
    Set {
        #[arg(help = "ID or URL Slug of the game. Example: [the-father] or [42]")]
        id: String,
    },
    #[command(about = "Removes the current game target")]
    Unset,
    #[command(about = "Prints the current clawdrop executable location")]
    Whereis,
    #[command(about = "Publish a post for the target or specified game")]
    Post {
        #[arg(long, help = "Game ID (if no target is set)")]
        id: Option<u64>,

        #[arg(long, help = "Title of the post")]
        title: String,

        #[arg(
            long,
            help = "Text body of the post or path to text file containing body"
        )]
        body: String,

        #[arg(long, help = "Path to optional cover image")]
        cover: Option<String>,

        #[arg(long, help = "Optional URL slug for the post")]
        slug: Option<String>,
    },
    // TODO: Read version from file.
    #[command(
        about = "Upload a new build of an existing game to Raccreative Games. The build must be unziped and ready to play"
    )]
    Push {
        /// Compact syntax: <id>:<os>/<exe>:<version> (example: 32:windows/game.exe:1.0.1)
        #[arg(
            help = "Positional shorthand syntax (id:os/exe:version)",
            required = false
        )]
        shorthand: Option<String>,

        #[arg(long, help = "The numeric id of the game, optional if target is set")]
        id: Option<u64>,

        #[arg(
            long,
            help = "Operating system for the build: windows | mac | linux | html"
        )]
        os: Option<String>,

        #[arg(
            long,
            help = "Name of the executable name (example: game.exe). Needed for manifest.json"
        )]
        exe: Option<String>,

        #[arg(
            long,
            help = "Version string (example: 1.0.1). If no version provided, it will auto bump unless --no-bump is set"
        )]
        version: Option<String>,

        #[arg(
            long,
            default_value = ".",
            help = "Path to build directory, default is current directory"
        )]
        path: String,

        #[arg(long, num_args = 1.., value_delimiter = ' ', help = "Ignore patterns (example: --ignore '*.json')")]
        ignore: Vec<String>,

        #[arg(
            long,
            help = "Prevent automatic version bump if no version is provided but target is configured"
        )]
        no_bump: bool,

        #[arg(
            long,
            help = "Ignore changes and upload everything in the build (except --ignore files)"
        )]
        force: bool,
    },
}
