#![feature(plugin)]

#[plugin]
#[macro_use]
#[no_link]
extern crate rustful_macros;

extern crate rustful;

use std::io::{File, IoResult};
use std::os::getenv;
use std::io::net::ip::{Port};
use std::sync::{Arc, RwLock};
use std::error::Error;
use std::str::FromStr;

use rustful::{Server, Context, Response, Handler, TreeRouter};
use rustful::cache::{CachedValue, CachedProcessedFile};
use rustful::Method::Get;
use rustful::StatusCode::InternalServerError;
use rustful::header::ContentType;

fn main() {
    println!("Visit http://localhost:8080 to try this example.");

    //Cache the page
    let page = Arc::new(CachedProcessedFile::new(Path::new("src/views/index.html"), None, read_string));

    //The shared counter state
    let value = Arc::new(RwLock::new(0));

    let router = insert_routes!{
        TreeRouter::new(): {
            "/" => Get: Profile{
                page: page.clone(),
                value: value.clone(),
                operation: None
            }
        }
    };

    let server_result = Server::new().port(get_server_port()).handlers(router).run();

    match server_result {
        Ok(_server) => {},
        Err(e) => println!("could not start server: {}", e.description())
    }
}

fn get_server_port() -> Port {
    getenv("PORT")
        .and_then(|s| FromStr::from_str(s.as_slice()))
        .unwrap_or(8080)
}

fn read_string(mut file: IoResult<File>) -> IoResult<Option<String>> {
    //Read file into a string
    file.read_to_string().map(|s| Some(s))
}


struct Profile {
    //We are using the handler to cache the page in this exmaple
    page: Arc<CachedProcessedFile<String>>,

    value: Arc<RwLock<i32>>,
    operation: Option<fn(i32) -> i32>
}

impl Handler for Profile {
    type Cache = ();

    fn handle_request(&self, _context: Context, mut response: Response) {
        self.operation.map(|o| {
            //Lock the value for writing and update it
            let mut value = self.value.write().unwrap();
            *value = (o)(*value);
        });

        response.set_header(ContentType(content_type!("text", "html", ("charset", "UTF-8"))));

        //Insert the value into the page and write it to the response
        match *self.page.borrow() {
            Some(ref page) => {
                let count = self.value.read().unwrap().to_string();

                try_send!(response.into_writer(), page.replace("{}", &count[]));
            },
            None => {
                //Oh no! The page was not loaded!
                response.set_status(InternalServerError);
            }
        }
    }
}
