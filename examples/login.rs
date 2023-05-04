#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;

use rouille::Request;
use rouille::Response;
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;

// This struct contains the data that we store on the server about each client.
#[derive(Debug, Clone)]
struct SessionData {
    login: String,
}

fn main() {
    println!("Now listening on localhost:8000");
    let sessions_storage: Mutex<HashMap<String, SessionData>> = Mutex::new(HashMap::new());

    rouille::start_server("0.0.0.0:8000", move |request| {
        rouille::log(&request, io::stdout(), || {
            rouille::session::session(request, "SID", 3600, |session| {
                let mut session_data = if session.client_has_sid() {
                    if let Some(data) = sessions_storage.lock().unwrap().get(session.id()) {
                        Some(data.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let response = handle_route(&request, &mut session_data);

                if let Some(d) = session_data {
                    sessions_storage
                        .lock()
                        .unwrap()
                        .insert(session.id().to_owned(), d);
                } else if session.client_has_sid() {
                    sessions_storage.lock().unwrap().remove(session.id());
                }

                response
            })
        })
    });
}

fn handle_route(request: &Request, session_data: &mut Option<SessionData>) -> Response {
    router!(request,
        (POST) (/login) => {

            let data = try_or_400!(post_input!(request, {
                login: String,
                password: String,
            }));

            println!("Login attempt with login {:?} and password {:?}", data.login, data.password);

            if data.password.starts_with("b") {
                *session_data = Some(SessionData { login: data.login });
                return Response::redirect_303("/");

            } else {
                return Response::html("Wrong login/password");
            }
        },

        _ => ()
    );

    if let Some(session_data) = session_data.as_ref() {
        // Logged in.
        handle_route_logged_in(request, session_data)
    } else {
        // Not logged in.
        router!(request,
            (GET) (/) => {
                Response::html(r#"
<html><head>
    <title>Login</title>
    <style>
      body {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
        margin: 0;
        background-color: #f4f7f9;
      }
      
      .login {
        background-color: #ffffff;
        width: 400px;
        box-shadow: 0 0 10px 0 rgba(0, 0, 0, 0.1);
        padding: 30px;
        text-align: center;
      }
      
      input[type="text"],
      input[type="password"] {
        border: none;
        background-color: #f7f7f7;
        padding: 12px 20px;
        margin: 8px 0;
        width: 100%;
        box-sizing: border-box;
        font-size: 16px;
        border-radius: 4px;
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
      }
      
      input[type="submit"] {
        background-color: #3385ff;
        border: none;
        color: #ffffff;
        text-transform: uppercase;
        padding: 15px 20px;
        margin-top: 16px;
        border-radius: 4px;
        font-size: 16px;
        cursor: pointer;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
      }
      
      input[type="submit"]:hover {
        background-color: #3385cc;
      }
      
      h2 {
        margin-top: 0;
        font-weight: normal;
        font-size: 28px;
      }
    </style>
  </head>
  <body>
    <div class="login">
      <h2>Xunlei Login</h2>
      <form action="/login" method="POST">
        <input type="text" id="username" name="username" placeholder="Enter your username">
        <br>
        <input type="password" id="password" name="password" placeholder="Enter your password">
        <br>
        <input type="submit" value="Login">
      </form>
    </div>
</body></html>
                "#)

            },

            _ => {
                // If the user tries to access any other route, redirect them to the login form.
                //
                // You may wonder: if I want to make some parts of my site public and some other
                // parts private, should I put all my public routes here? The answer is no. The way
                // this example is structured is appropriate for a website that is entirely
                // private. Don't hesitate to structure it in a different way, for example by
                // having a function that is dedicated only to public routes.
                Response::redirect_303("/")
            }
        )
    }
}

// This function handles the routes that are accessible only if the user is logged in.
fn handle_route_logged_in(request: &Request, _session_data: &SessionData) -> Response {
    router!(request,
        (GET) (/) => {
            // Show some greetings with a dummy response.
            Response::html(r#"You are now logged in. If you close your tab and open it again,
                              you will still be logged in.<br />
                              <a href="/private">Click here for the private area</a>
                              <form action="/logout" method="POST">
                              <button>Logout</button></form>"#)
        },

        (GET) (/private) => {
            // This route is here to demonstrate that the client can go to `/private` only if
            // they are successfully logged in.
            Response::html(r#"You are in the private area! <a href="/">Go back</a>."#)
        },

        _ => Response::empty_404()
    )
}
