use std::{fs, path::PathBuf};

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    delete, get,
    http::StatusCode,
    post, put,
    web::{self, Data, Json},
    App, HttpRequest, HttpServer, Responder, Result,
};

mod conf;
mod db;
use db::UserTasksDB;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ResponseTask {
    task_id: i64,
    task_title: String,
    task_description: String,
}

#[derive(Serialize)]
struct Response {
    user_id: i64,
    username: String,
    tasks: Vec<ResponseTask>,
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct SessionInfo {
    user_id: i64,
    username: String,
}

#[derive(Serialize, Deserialize)]
struct TaskInfo {
    task_id: i64,
    task_title: String,
    task_description: String,
}

#[get("/data")]
async fn data(user_tasks_db: Data<UserTasksDB>, session: Session) -> impl Responder {
    match session.get::<SessionInfo>("session_id") {
        Err(err) => {
            println!("{err}");
            Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "SessionGetError".to_string(),
            })
            .customize()
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(result) => match result {
            Some(session_data) => {
                let mut tasks: Vec<ResponseTask> = vec![];
                for task in user_tasks_db.get_tasks_by_user_id(session_data.user_id) {
                    tasks.push(ResponseTask {
                        task_id: task.task_id,
                        task_title: task.title,
                        task_description: task.description,
                    })
                }

                Json(Response {
                    user_id: session_data.user_id,
                    username: session_data.username.to_string(),
                    tasks: tasks,
                    success: true,
                    message: "User logged in!".to_string(),
                })
                .customize()
            }
            None => Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: true,
                message: "Not logged in!".to_string(),
            })
            .customize(),
        },
    }
}

#[post("/login")]
async fn login(
    user_tasks_db: Data<UserTasksDB>,
    login_info: web::Json<LoginInfo>,
    session: Session,
) -> impl Responder {
    let users = user_tasks_db.get_user_by_credentials(&login_info.username, &login_info.password);
    let mut response = Response {
        user_id: -1,
        username: "Anon".to_string(),
        tasks: vec![],
        success: false,
        message: "Login: No User found!".to_string(),
    };
    let mut status_code = StatusCode::BAD_REQUEST;
    if users.len() == 0 {
    } else if users.len() == 1 {
        let session_info = SessionInfo {
            user_id: users[0].user_id,
            username: users[0].username.clone(),
        };
        let _ = session.insert::<SessionInfo>("session_id", session_info);
        response.user_id = users[0].user_id;
        response.username = users[0].username.clone();
        let mut tasks: Vec<ResponseTask> = vec![];
        for task in user_tasks_db.get_tasks_by_user_id(users[0].user_id) {
            tasks.push(ResponseTask {
                task_id: task.task_id,
                task_title: task.title,
                task_description: task.description,
            })
        }
        response.tasks = tasks;
        response.success = true;
        response.message = "Logged in successfully!".to_string();
        status_code = StatusCode::OK;
    } else {
        response.message = "DB fail more than 1 user".to_string();
        status_code = StatusCode::INTERNAL_SERVER_ERROR;
    }
    Json(response).customize().with_status(status_code)
}

#[delete("/logout")]
async fn logout(session: Session) -> impl Responder {
    match session.get::<SessionInfo>("session_id") {
        Err(err) => {
            println!("{err}");
            Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Logout : SessionGetError".to_string(),
            })
            .customize()
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(result) => match result {
            Some(session_data) => {
                session.clear();
                Json(Response {
                    user_id: -1,
                    username: "Anon".to_string(),
                    tasks: vec![],
                    success: true,
                    message: format!("Logged out successfully"),
                })
                .customize()
            }
            None => Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Already logged out!".to_string(),
            })
            .customize(),
        },
    }
}

#[post("/task")]
async fn task_create(
    user_tasks_db: Data<UserTasksDB>,
    task_info: web::Json<TaskInfo>,
    session: Session,
) -> impl Responder {
    match session.get::<SessionInfo>("session_id") {
        Err(err) => {
            println!("{err}");
            Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Create task : SessionGetError".to_string(),
            })
            .customize()
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(result) => match result {
            Some(session_data) => {
                let success = user_tasks_db.create_task(
                    session_data.user_id,
                    task_info.task_title.clone(),
                    task_info.task_description.clone(),
                );

                let mut tasks: Vec<ResponseTask> = vec![];
                for task in user_tasks_db.get_tasks_by_user_id(session_data.user_id) {
                    tasks.push(ResponseTask {
                        task_id: task.task_id,
                        task_title: task.title,
                        task_description: task.description,
                    })
                }
                Json(Response {
                    user_id: session_data.user_id,
                    username: session_data.username.to_string(),
                    tasks: tasks,
                    success: success,
                    message: format!(
                        "Create task: {}!",
                        if success { "successful" } else { "failed" }
                    ),
                })
                .customize()
                .with_status(if success {
                    StatusCode::OK
                } else {
                    StatusCode::BAD_REQUEST
                })
            }
            None => Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Create task: unauthorized!".to_string(),
            })
            .customize()
            .with_status(StatusCode::UNAUTHORIZED),
        },
    }
}

#[put("/task")]
async fn task_update(
    user_tasks_db: Data<UserTasksDB>,
    task_info: web::Json<TaskInfo>,
    session: Session,
) -> impl Responder {
    match session.get::<SessionInfo>("session_id") {
        Err(err) => {
            println!("{err}");
            Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Update task : SessionGetError".to_string(),
            })
            .customize()
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(result) => match result {
            Some(session_data) => {
                let success = user_tasks_db.update_task(
                    task_info.task_id,
                    session_data.user_id,
                    task_info.task_title.clone(),
                    task_info.task_description.clone(),
                );

                let mut tasks: Vec<ResponseTask> = vec![];
                for task in user_tasks_db.get_tasks_by_user_id(session_data.user_id) {
                    tasks.push(ResponseTask {
                        task_id: task.task_id,
                        task_title: task.title,
                        task_description: task.description,
                    })
                }
                Json(Response {
                    user_id: session_data.user_id,
                    username: session_data.username.to_string(),
                    tasks: tasks,
                    success: success,
                    message: format!(
                        "Update task: {}!",
                        if success { "successful" } else { "failed" }
                    ),
                })
                .customize()
                .with_status(if success {
                    StatusCode::OK
                } else {
                    StatusCode::BAD_REQUEST
                })
            }
            None => Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Update task: unauthorized!".to_string(),
            })
            .customize()
            .with_status(StatusCode::UNAUTHORIZED),
        },
    }
}

#[delete("/task")]
async fn task_delete(
    user_tasks_db: Data<UserTasksDB>,
    task_info: web::Json<TaskInfo>,
    session: Session,
) -> impl Responder {
    match session.get::<SessionInfo>("session_id") {
        Err(err) => {
            println!("{err}");
            Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Delete task : SessionGetError".to_string(),
            })
            .customize()
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(result) => match result {
            Some(session_data) => {
                let success = user_tasks_db.delete_task(task_info.task_id, session_data.user_id);

                let mut tasks: Vec<ResponseTask> = vec![];
                for task in user_tasks_db.get_tasks_by_user_id(session_data.user_id) {
                    tasks.push(ResponseTask {
                        task_id: task.task_id,
                        task_title: task.title,
                        task_description: task.description,
                    })
                }
                Json(Response {
                    user_id: session_data.user_id,
                    username: session_data.username.to_string(),
                    tasks: tasks,
                    success: success,
                    message: format!(
                        "Delete task: {}!",
                        if success { "successful" } else { "failed" }
                    ),
                })
                .customize()
                .with_status(if success {
                    StatusCode::OK
                } else {
                    StatusCode::BAD_REQUEST
                })
            }
            None => Json(Response {
                user_id: -1,
                username: "Anon".to_string(),
                tasks: vec![],
                success: false,
                message: "Delete task: unauthorized!".to_string(),
            })
            .customize()
            .with_status(StatusCode::UNAUTHORIZED),
        },
    }
}

async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req
        .match_info()
        .query("../dist/index.html")
        .parse()
        .unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // init db
    let mut user_tasks_db: UserTasksDB = UserTasksDB::new();
    user_tasks_db.reset();

    let secret_key = Key::from(conf::SECRET_KEY);

    // start server
    HttpServer::new(
        move || {
            let cors = Cors::permissive();
            App::new()
                .wrap(cors)
                .wrap(SessionMiddleware::new(
                    CookieSessionStore::default(),
                    secret_key.clone(),
                ))
                .app_data(Data::new(UserTasksDB::new()))
                .service(data)
                .service(login)
                .service(logout)
                .service(task_create)
                .service(task_update)
                .service(task_delete)
                .service(actix_files::Files::new("/", ".").index_file("index.html"))
        }, // login route
    )
    .bind(("127.0.0.1", conf::PORT))?
    .run()
    .await
}
