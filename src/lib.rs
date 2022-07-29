

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Balance, Promise};
use serde::Serialize;
use serde::Deserialize;


near_sdk::setup_alloc!();

pub const VAULT_FEE: u128 = 500;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Users {
    users_id: AccountId,
    email: String,
    name: String,
    number: String,
    edad: i8,
    work: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ServicesPrices {
    id_servicio: i128,
    user_id: AccountId,
    number: String,
    nombre_servicio: String,
    price: i32,
    status: i8,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    id_servicio: i128,
    users: Vec<Users>,
    servicios: Vec<ServicesPrices>,
    
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "El programa ya ha sido iniciado");
        Self {
            id_servicio: 0,
            servicios: Vec::new(),
            users: Vec::new(),
            
           
        }
    }    

    pub fn set_user(&mut self, 
        users_id: AccountId,
        email: String,
        name: String,
        number: String,
        edad: i8,
        work: bool
    ){

        let user = self.users.iter().find(|user| user.users_id == users_id.to_string());
        if user.is_some() {
            env::panic(b"Este perfil ya existe");
        }
        let data = Users {
            users_id: users_id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            number: number.to_string(),
            edad: edad,
            work: work,
        };
        self.users.push(data);
        env::log(b"Usuario creado");
    }

    pub fn put_users(&mut self, users_id: AccountId, email: String, name: String, number: String,
        edad: i8) {
        let index = self.users.iter().position(|x| x.users_id == users_id.to_string()).expect("Este usuario no existe");
        self.users[index].email = email.to_string();
        self.users[index].name = name.to_string();
        self.users[index].number = number.to_string();
        self.users[index].edad = edad;

        env::log(b"Usuario actualizado");

    }

    pub fn get_users(&self, users_id: Option<AccountId>) -> Vec<Users> {
        let mut users = self.users.clone();

        if users_id.is_some() {
            users = self.users.iter().filter(|x| x.users_id.to_string() == users_id.clone().unwrap()).map(|x| Users {
                users_id: x.users_id.clone(),
                email: x.email.to_string(),
                name: x.name.to_string(),
                number: x.number.to_string(),
                edad: x.edad,
                work: x.work,
            }).collect();
        }
        users
    }

    pub fn set_services(&mut self, 
        user_id: AccountId,
        nombre_servicio: String,
        price: i32
    ){
        let user = self.users.iter().find(|x| x.users_id == user_id).expect("No se encuentra registrado");
        if user.work == true {
            self.id_servicio += 1;
            let data = ServicesPrices {
                id_servicio: self.id_servicio,
                user_id: user_id.to_string(),
                number: user.number.clone(),
                nombre_servicio: nombre_servicio.to_string(),
                price: price,
                status: 1
            };
            self.servicios.push(data);
            env::log(b"Servicio creado");
        } else {
            env::panic(b"Usted no es trabajador")
        }
        
    }

    pub fn get_servicios(&self, nombre_servicio: Option<String>) -> Vec<ServicesPrices> {
        let mut services = self.servicios.clone();

        if nombre_servicio.is_some() {
            services = self.servicios.iter().filter(|x| x.nombre_servicio.to_string() == nombre_servicio.clone().unwrap()).map(|x| ServicesPrices {
                id_servicio: x.id_servicio.clone(),
                user_id: x.user_id.clone(),
                number: x.number.clone(),
                price:x.price.clone(),
                nombre_servicio: x.nombre_servicio.clone(),
                status: x.status.clone()
            }).collect();
        }
        services
    }

    #[payable]
    pub fn pagar_servicio(
        &mut self, 
        id_servicio: i128,
    ) -> ServicesPrices {
        let initial_storage_usage = env::storage_usage();
        let services = self.servicios.iter().find(|x| x.id_servicio == id_servicio).expect("No se encuentra el servicio");

        if services.status == 1 {
            let price: u128 = services.price as u128;
            let attached_deposit = env::attached_deposit();

            assert!(
                attached_deposit >= price,
                "El deposito en menor que el precio : {}",
                price
            );

            Promise::new(services.user_id.clone()).transfer(price);
            
            refund_deposit(env::storage_usage() - initial_storage_usage, price);

            let index = self.servicios.iter().position(|x| x.id_servicio == id_servicio).expect("Este servicio no se encuentra");
            self.servicios[index].status = 2;
            
            self.servicios[index].clone()
        } else {
            env::panic(b"Servicio ya pagado");
        }
    }
}

fn refund_deposit(storage_used: u64, extra_spend: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit() - extra_spend;

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}