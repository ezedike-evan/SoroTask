#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
    Val, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidInterval = 1,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TaskConfig {
    pub creator: Address,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub resolver: Option<Address>,
    pub interval: u64,
    pub last_run: u64,
    pub gas_balance: i128,
}

#[contracttype]
pub enum DataKey {
    Task(u64),
    Counter,
}

pub trait ResolverInterface {
    fn check_condition(env: Env, args: Vec<Val>) -> bool;
}

#[contract]
pub struct SoroTaskContract;

#[contractimpl]
impl SoroTaskContract {
    /// Registers a new task in the marketplace.
    /// Returns the unique sequential ID of the registered task.
    pub fn register(env: Env, config: TaskConfig) -> u64 {
        // Ensure the creator has authorized the registration
        config.creator.require_auth();

        // Validate the task interval
        if config.interval == 0 {
            panic_with_error!(&env, Error::InvalidInterval);
        }

        // Generate a unique sequential ID
        let mut counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Counter)
            .unwrap_or(0);
        counter += 1;
        env.storage().persistent().set(&DataKey::Counter, &counter);

        // Store the task configuration
        env.storage()
            .persistent()
            .set(&DataKey::Task(counter), &config);

        // Emit TaskRegistered event
        env.events().publish(
            (Symbol::new(&env, "TaskRegistered"), counter),
            config.creator.clone(),
        );

        counter
    }

    /// Retrieves a task configuration by its ID.
    pub fn get_task(env: Env, task_id: u64) -> Option<TaskConfig> {
        env.storage().persistent().get(&DataKey::Task(task_id))
    }

    pub fn monitor(_env: Env) {
        // TODO: Implement task monitoring logic
    }

    /// Executes a task if its conditions are met.
    pub fn execute(env: Env, task_id: u64) {
        let task_key = DataKey::Task(task_id);
        let mut config: TaskConfig = env
            .storage()
            .persistent()
            .get(&task_key)
            .expect("Task not found");

        if env.ledger().timestamp() < config.last_run + config.interval {
            return;
        }

        let should_execute = match config.resolver {
            Some(ref resolver_address) => {
                // Call standardized method check_condition(args) -> bool
                // Use try_invoke_contract to handle failure/revert gracefully
                match env.try_invoke_contract::<bool, soroban_sdk::Error>(
                    resolver_address,
                    &Symbol::new(&env, "check_condition"),
                    config.args.clone(),
                ) {
                    Ok(Ok(result)) => result,
                    _ => false, // Failure or non-true result means don't proceed
                }
            }
            None => true,
        };

        if should_execute {
            // Execute the target function
            env.invoke_contract::<Val>(&config.target, &config.function, config.args.clone());

            // Update last_run
            config.last_run = env.ledger().timestamp();
            env.storage().persistent().set(&task_key, &config);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events, Ledger as _};
    use soroban_sdk::{contract, contractimpl, vec, Env, FromVal, IntoVal};

    #[contract]
    pub struct DummyContract;

    #[contractimpl]
    impl DummyContract {
        pub fn hello(_env: Env) {}
    }

    #[test]
    fn test_register_and_get() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, SoroTaskContract);
        let client = SoroTaskContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let target = Address::generate(&env);

        let config = TaskConfig {
            creator: creator.clone(),
            target: target.clone(),
            function: Symbol::new(&env, "hello"),
            args: vec![&env, 0i128.into_val(&env)],
            resolver: None,
            interval: 3600,
            last_run: 0,
            gas_balance: 1000,
        };

        let task_id = client.register(&config);
        assert_eq!(task_id, 1);

        let retrieved_config = client.get_task(&task_id).unwrap();
        assert_eq!(retrieved_config.creator, config.creator);
        assert_eq!(retrieved_config.target, config.target);
        assert_eq!(retrieved_config.function, config.function);
        assert_eq!(retrieved_config.interval, config.interval);
        assert_eq!(retrieved_config.gas_balance, config.gas_balance);

        // Check event
        let events = env.events().all();
        let last_event = events.last().unwrap();

        assert_eq!(last_event.0, contract_id);

        let topics = last_event.1.clone();
        assert_eq!(
            Symbol::from_val(&env, &topics.get(0).unwrap()),
            Symbol::new(&env, "TaskRegistered")
        );
        assert_eq!(u64::from_val(&env, &topics.get(1).unwrap()), 1u64);

        let data: Address = last_event.2.clone().into_val(&env);
        assert_eq!(data, creator);
    }

    #[test]
    fn test_sequential_ids() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, SoroTaskContract);
        let client = SoroTaskContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let target = Address::generate(&env);

        let config = TaskConfig {
            creator: creator.clone(),
            target: target.clone(),
            function: Symbol::new(&env, "hello"),
            args: vec![&env],
            resolver: None,
            interval: 3600,
            last_run: 0,
            gas_balance: 1000,
        };

        let id1 = client.register(&config);
        let id2 = client.register(&config);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_register_invalid_interval() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, SoroTaskContract);
        let client = SoroTaskContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let target = Address::generate(&env);

        let config = TaskConfig {
            creator: creator.clone(),
            target: target.clone(),
            function: Symbol::new(&env, "hello"),
            args: vec![&env],
            resolver: None,
            interval: 0, // Invalid
            last_run: 0,
            gas_balance: 1000,
        };

        let result = client.try_register(&config);
        assert_eq!(result, Err(Ok(soroban_sdk::Error::from_contract_error(1))));
    }

    #[test]
    fn test_execute_honors_interval() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, SoroTaskContract);
        let client = SoroTaskContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let dummy_id = env.register_contract(None, DummyContract);
        let target = dummy_id.clone();

        let config = TaskConfig {
            creator: creator.clone(),
            target: target.clone(),
            function: Symbol::new(&env, "hello"),
            args: Vec::new(&env),
            resolver: None,
            interval: 100,
            last_run: 0,
            gas_balance: 1000,
        };

        let task_id = client.register(&config);

        // First execution (ledger 50, last_run 0, interval 100)
        // 50 < 0 + 100 -> returns early
        env.ledger().set_timestamp(50);
        client.execute(&task_id);
        assert_eq!(client.get_task(&task_id).unwrap().last_run, 0);

        env.ledger().set_timestamp(150);
        client.execute(&task_id);
        assert_eq!(client.get_task(&task_id).unwrap().last_run, 150);

        // Next execution too soon
        env.ledger().set_timestamp(200);
        client.execute(&task_id);
        assert_eq!(client.get_task(&task_id).unwrap().last_run, 150);
    }
}

