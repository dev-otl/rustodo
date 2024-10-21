use std::fmt::Display;

use sqlite::State;

pub struct User {
    pub user_id: i64,
    pub username: String,
    password: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.user_id, self.username, self.password)
    }
}

pub struct Task {
    pub task_id: i64,
    pub title: String,
    pub description: String,
}

pub struct UserTasksDB {
    connection: sqlite::Connection,
}

impl Clone for UserTasksDB {
    fn clone(&self) -> Self {
        Self {
            connection: sqlite::open("db.sql").unwrap(),
        }
    }
}

impl UserTasksDB {
    pub fn new() -> UserTasksDB {
        UserTasksDB {
            connection: sqlite::open("db.sql").unwrap(),
        }
    }

    pub fn reset(&mut self) {
        let query: &str = "
        DROP TABLE   IF EXISTS users;
        CREATE TABLE users (
            user_id INTEGER NOT NULL UNIQUE, 
            username TEXT NOT NULL UNIQUE, 
            password TEXT NOT NULL,
            PRIMARY KEY('user_id' AUTOINCREMENT)
        );
        INSERT INTO users VALUES(0, 'user0', 'password0');
        INSERT INTO users VALUES(NULL, 'user1', 'password1');
        INSERT INTO users VALUES(NULL, 'user2', 'password2'); 
    
    
        DROP TABLE IF EXISTS tasks;
        CREATE TABLE tasks (
            task_id INTEGER NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL UNIQUE,
            description TEXT,
            PRIMARY KEY('task_id' AUTOINCREMENT),
            FOREIGN KEY('user_id') REFERENCES users('user_id')
        );
        INSERT INTO tasks VALUES (NULL, 1, 'title 11', 'description 11');
        INSERT INTO tasks VALUES (NULL, 2, 'title 21', 'description 21');
        INSERT INTO tasks VALUES (NULL, 2, 'title 31', 'description 31');
    
        ";

        self.connection.execute(query).unwrap();
    }

    pub fn get_user_by_credentials(&self, username: &str, password: &str) -> Vec<User> {
        let query = "SELECT * from users WHERE username = ? AND password = ? ;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind(&[(1, username), (2, password)][..]).unwrap();

        let mut users: Vec<User> = vec![];
        while let Ok(State::Row) = statement.next() {
            users.push(User {
                user_id: statement.read::<i64, _>("user_id").unwrap(),
                username: statement.read::<String, _>("username").unwrap(),
                password: statement.read::<String, _>("password").unwrap(),
            });
        }
        return users;
    }

    pub fn get_tasks_by_user_id(&self, user_id: i64) -> Vec<Task> {
        let query = "SELECT * from tasks WHERE user_id = ? ;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, user_id)).unwrap();

        let mut tasks: Vec<Task> = vec![];
        while let Ok(State::Row) = statement.next() {
            tasks.push(Task {
                task_id: statement.read::<i64, _>("task_id").unwrap(),
                title: statement.read::<String, _>("title").unwrap(),
                description: statement.read::<String, _>("description").unwrap(),
            });
        }
        return tasks;
    }

    pub fn create_task(&self, user_id: i64, title: String, description: String) -> bool {
        let query = format!(
            "INSERT INTO tasks VALUES (NULL, {}, '{}', '{}');",
            user_id, title, description
        );
        let result = &self.connection.execute(query);
        match result {
            Ok(_) => true,
            Err(err) => {
                println!("{err}");
                false
            }
        }
    }

    pub fn update_task(
        &self,
        task_id: i64,
        user_id: i64,
        title: String,
        description: String,
    ) -> bool {
        let query = format!(
            "UPDATE tasks 
            SET title = '{}', description = '{}'
            WHERE task_id = {} AND user_id = {};",
            title, description, task_id, user_id
        );
        //println!("{query}");
        let result = &self.connection.execute(query);
        match result {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn delete_task(&self, task_id: i64, user_id: i64) -> bool {
        let query = format!(
            "DELETE FROM tasks WHERE task_id = {} AND user_id = {};",
            task_id, user_id
        );
        //println!("{query}");
        let result = &self.connection.execute(query);
        match result {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
