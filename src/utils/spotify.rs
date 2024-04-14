/////////////////////////////////////////
// rspotify Client-related Utility Methods
////////////////////////////////////////

use std::env;
use std::io::{self, Write};

use rspotify::{scopes, AuthCodeSpotify, ClientError, Config, Credentials, OAuth, Token};
use rspotify::clients::{BaseClient, OAuthClient};

///
/// Creates a new AuthCodeSpotify client with the given credentials and oauth.
pub fn get_client() -> AuthCodeSpotify {
    let config = Config {
        token_cached: true,
        ..Default::default()
    };

    let credentials: Credentials = match Credentials::from_env() {
        Some(_) => {
            Credentials::from_env().unwrap()
        },
        None => {
            id_secret_prompt().unwrap()
        }
    };

    let oauth: OAuth = OAuth {
        redirect_uri: "http://localhost:8000/callback".to_string(),
        scopes: scopes!(
            "user-read-playback-state",
            "user-read-currently-playing"
        ),
        ..Default::default()
        
    };

    AuthCodeSpotify::with_config(credentials, oauth, config)
}

///
/// Prompts user for Spotify client id and secret, then sets it to ENV VAR.
/// 
/// returns: Option<Credentials> - the client id and secret in Credentials object
pub fn id_secret_prompt() -> Option<Credentials> {
    println!("RSPOTIFY_CLIENT_ID/RSPOTIFY_CLIENT_SECRET not found in environment");
    print!("Enter RSPOTIFY_CLIENT_ID: ");
    let _ = io::stdout().flush();
    let mut client_id = String::new();
    io::stdin().read_line(&mut client_id).expect("Unable to read RSPOTIFY_CLIENT_ID");

    print!("Enter RSPOTIFY_CLIENT_SECRET: ");
    let mut client_secret = String::new();
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut client_secret).expect("Unable to read RSPOTIFY_CLIENT_SECRET");

    if client_id.len() == 0 || client_secret.len() == 0 {
        panic!("RSPOTIFY_CLIENT_ID and RSPOTIFY_CLIENT_SECRET cannot be empty!")
    }

    // persist client id/secret to env var
    env::set_var("RSPOTIFY_CLIENT_ID", client_id.clone());
    env::set_var("RSPOTIFY_CLIENT_SECRET", client_secret.clone());

    Some(Credentials::new(client_id.trim().as_ref(), client_secret.trim().as_ref()))
}

pub fn get_token(client: &AuthCodeSpotify) -> Option<Token> {
    client.get_token().lock().unwrap().clone()
}

pub fn get_authorize_url(client: &AuthCodeSpotify) -> String {
    client.get_authorize_url(false).unwrap()
}

pub fn get_access_token(client: &AuthCodeSpotify, code: &str) -> Result<(), ClientError> {
    client.request_token(code)
}