use minijinja::Environment;

mod passwordstore;

/// Add custom filters that provide integration for password managers.
pub fn add_password_manager_functions(env: &mut Environment) {
    env.add_function("pass", passwordstore::pass);
}
