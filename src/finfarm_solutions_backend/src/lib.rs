#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::api::time;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

/// Define a struct for Fish data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Fish {
    id: u64,
    species: String,
    description: String,
    quantity: u64,
    age_in_months: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

/// Define a struct for Tank data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Tank {
    id: u64,
    tank_name: String,
    capacity_liters: u64,
    current_stock: Vec<Fish>,
    created_at: u64,
    updated_at: Option<u64>,
}

/// Define a struct for Farm data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Farm {
    id: u64,
    farm_name: String,
    location: String,
    tanks: Vec<Tank>,
    created_at: u64,
    updated_at: Option<u64>,
}

/// Implement Storable and BoundedStorable traits for Fish struct
impl Storable for Fish {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Fish {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

/// Implement Storable and BoundedStorable traits for Tank struct
impl Storable for Tank {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Tank {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

/// Implement Storable and BoundedStorable traits for Farm struct
impl Storable for Farm {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Farm {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread memory manager
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static FISH_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static TANK_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a counter")
    );

    static FARM_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))), 0)
            .expect("Cannot create a counter")
    );

    static FISH_STORAGE: RefCell<StableBTreeMap<u64, Fish, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );

    static TANK_STORAGE: RefCell<StableBTreeMap<u64, Tank, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );

    static FARM_STORAGE: RefCell<StableBTreeMap<u64, Farm, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))))
    );
}

/// Fish Payload data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct FishPayload {
    species: String,
    description: String,
    quantity: u64,
    age_in_months: u64,
}

/// Tank Payload data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct TankPayload {
    tank_name: String,
    capacity_liters: u64,
}

/// Farm Payload data
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct FarmPayload {
    farm_name: String,
    location: String,
}

/// Creates a new fish with the provided payload.
#[ic_cdk::update]
fn create_fish(fish_payload: FishPayload) -> Result<Fish, Error> {
    // Payload Validation
    if fish_payload.species.is_empty()
        || fish_payload.description.is_empty()
        || fish_payload.quantity == 0
        || fish_payload.age_in_months == 0
    {
        return Err(Error::NotFound {
            msg: format!("Invalid Fish Payload. Should not be empty or zero"),
        });
    }
    let id = FISH_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");

    let fish = Fish {
        id,
        species: fish_payload.species,
        description: fish_payload.description,
        quantity: fish_payload.quantity,
        age_in_months: fish_payload.age_in_months,
        created_at: time(),
        updated_at: None,
    };

    FISH_STORAGE.with(|s| s.borrow_mut().insert(id, fish.clone()));

    Ok(fish)
}

/// Updates a fish with the provided payload.
#[ic_cdk::update]
fn update_fish(fish_id: u64, fish_payload: FishPayload) -> Result<Fish, Error> {
    // Payload Validation
    if fish_payload.species.is_empty()
        || fish_payload.description.is_empty()
        || fish_payload.quantity == 0
        || fish_payload.age_in_months == 0
    {
        return Err(Error::NotFound {
            msg: format!("Invalid Fish Payload. Should not be empty or zero"),
        });
    }
    match FISH_STORAGE.with(|service| service.borrow().get(&fish_id)) {
        Some(mut fish) => {
            fish.species = fish_payload.species;
            fish.description = fish_payload.description;
            fish.quantity = fish_payload.quantity;
            fish.age_in_months = fish_payload.age_in_months;
            fish.updated_at = Some(time());
            FISH_STORAGE.with(|m| m.borrow_mut().insert(fish_id, fish.clone()));
            Ok(fish)
        }
        None => Err(Error::NotFound {
            msg: format!("Fish with id={} not found", fish_id),
        }),
    }
}

/// Batch creates fish with the provided payloads.
#[ic_cdk::update]
fn batch_create_fish(fish_payload: Vec<FishPayload>) -> Result<Vec<Fish>, Error> {
    let mut fish_vec = Vec::new();
    for fish in fish_payload.iter() {
        let id = FISH_ID_COUNTER
            .with(|counter| {
                let current_value = *counter.borrow().get();
                counter.borrow_mut().set(current_value + 1)
            })
            .expect("Cannot increment id counter");

        let fish = Fish {
            id,
            species: fish.species.clone(),
            description: fish.description.clone(),
            quantity: fish.quantity,
            age_in_months: fish.age_in_months,
            created_at: time(),
            updated_at: None,
        };

        FISH_STORAGE.with(|s| s.borrow_mut().insert(id, fish.clone()));
        fish_vec.push(fish);
    }
    Ok(fish_vec)
}

// fish age update utility
#[ic_cdk::update]
fn update_fish_age(fish_id: u64, age_in_months: u64) -> Result<Fish,Error> {
    match FISH_STORAGE.with(|service| service.borrow().get(&fish_id)) {
        Some(mut fish) => {
            fish.age_in_months = age_in_months;
            fish.updated_at = Some(time());
            FISH_STORAGE.with(|m| m.borrow_mut().insert(fish_id, fish.clone()));
            Ok(fish)
        }
        None => Err(Error::NotFound {
            msg: format!("fish with id={} not found", fish_id),
        }),
    }
}

// get a fish by id
#[ic_cdk::query]
fn get_fish(fish_id: u64) -> Result<Fish,Error> {
    FISH_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&fish_id)
            .ok_or(Error::NotFound {
                msg: format!("Fish with id={} not found", fish_id),
            })
    })
}

// get all fish 
#[ic_cdk::query]
fn get_all_fish() -> Vec<Fish> {
    FISH_STORAGE.with(|service| {
        service
            .borrow_mut()
            .iter()
            .map(|(_, fish)| fish.clone())
            .collect()
    })
}

// delete a fish by id
#[ic_cdk::update]
fn delete_fish(fish_id: u64) -> Result<(),Error> {
    match FISH_STORAGE.with(|service| service.borrow_mut().remove(&fish_id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("fish with id={} not found", fish_id),
        }),
    }
}

// create a new tank
#[ic_cdk::update]
fn create_tank(tank_payload: TankPayload) -> Result<Tank,Error> {
    // payload Validation all in one line
    if tank_payload.tank_name.is_empty() || tank_payload.capacity_liters == 0 {
        return Err(Error::NotFound {
            msg: format!("Invalid Tank Payload. Should not be empty or zero"),
        });
    }   
    let id = TANK_ID_COUNTER
    .with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    })
    .expect("cannot increment id counter");

    let tank = Tank {
        id,
        tank_name: tank_payload.tank_name,
        capacity_liters: tank_payload.capacity_liters,
        current_stock: Vec::new(),
        created_at: time(),
        updated_at: None,
    };

    TANK_STORAGE.with(|s| s.borrow_mut().insert(id, tank.clone()));

    Ok(tank)
}

// update a tank
#[ic_cdk::update]
fn update_tank(tank_id: u64, tank_payload: TankPayload) -> Result<Tank,Error> {
    // payload Validation all in one line
    if tank_payload.tank_name.is_empty() || tank_payload.capacity_liters == 0 {
        return Err(Error::NotFound {
            msg: format!("Invalid Tank Payload. Should not be empty or zero"),
        });
    }   
    match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
        Some(mut tank) => {
            tank.tank_name = tank_payload.tank_name;
            tank.capacity_liters = tank_payload.capacity_liters;
            tank.updated_at = Some(time());
            TANK_STORAGE.with(|m| m.borrow_mut().insert(tank_id, tank.clone()));
            Ok(tank)
        }
        None => Err(Error::NotFound {
            msg: format!("tank with id={} not found", tank_id),
        }),
    }

}

// insert fish into tank
#[ic_cdk::update]
fn insert_fish_into_tank(tank_id: u64, fish_id: u64) -> Result<Tank,Error> {
    match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
        Some(mut tank) => {
            match FISH_STORAGE.with(|service| service.borrow().get(&fish_id)) {
                Some(fish) => {
                    tank.current_stock.push(fish.clone());
                    tank.updated_at = Some(time());
                    TANK_STORAGE.with(|m| m.borrow_mut().insert(tank_id, tank.clone()));
                    Ok(tank)
                }
                None => Err(Error::NotFound {
                    msg: format!("fish with id={} not found", fish_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("tank with id={} not found", tank_id),
        }),
    }
}

//remove fish from tank
#[ic_cdk::update]
fn remove_fish_from_tank(tank_id: u64, fish_id: u64) -> Result<Tank,Error> {
    match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
        Some(mut tank) => {
            match FISH_STORAGE.with(|service| service.borrow().get(&fish_id)) {
                Some(fish) => {
                    let index = tank.current_stock.iter().position(|x| x.id == fish.id).unwrap();
                    tank.current_stock.remove(index);
                    tank.updated_at = Some(time());
                    TANK_STORAGE.with(|m| m.borrow_mut().insert(tank_id, tank.clone()));
                    Ok(tank)
                }
                None => Err(Error::NotFound {
                    msg: format!("fish with id={} not found", fish_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("tank with id={} not found", tank_id),
        }),
    }
}

// check tank capacity
#[ic_cdk::query]
fn check_tank_capacity(tank_id: u64) -> Result<u64,Error> {
    match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
        Some(tank) => {
            let mut total_capacity = 0;
            for fish in tank.current_stock.iter() {
                total_capacity += fish.quantity;
            }
            Ok(total_capacity)
        }
        None => Err(Error::NotFound {
            msg: format!("tank with id={} not found", tank_id),
        }),
    }
}

// get a tank by id
#[ic_cdk::query]
fn get_tank(tank_id: u64) -> Result<Tank,Error> {
    TANK_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&tank_id)
            .ok_or(Error::NotFound {
                msg: format!("Tank with id={} not found", tank_id),
            })
    })
}

// get all tanks
#[ic_cdk::query]
fn get_all_tanks() -> Vec<Tank> {
    TANK_STORAGE.with(|service| {
        service
            .borrow_mut()
            .iter()
            .map(|(_, tank)| tank.clone())
            .collect()
    })
}

// delete a tank by id
#[ic_cdk::update]
fn delete_tank(tank_id: u64) -> Result<(),Error> {
    match TANK_STORAGE.with(|service| service.borrow_mut().remove(&tank_id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("tank with id={} not found", tank_id),
        }),
    }
}

// create a new farm
#[ic_cdk::update]
fn create_farm(farm_payload: FarmPayload) -> Result<Farm,Error> {
    // payload Validation all in one line
    if farm_payload.farm_name.is_empty() || farm_payload.location.is_empty() {
        return Err(Error::NotFound {
            msg: format!("Invalid Farm Payload. Should not be empty or zero"),
        });
    }   
    let id = FARM_ID_COUNTER
    .with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    })
    .expect("cannot increment id counter");

    let farm = Farm {
        id,
        farm_name: farm_payload.farm_name,
        location: farm_payload.location,
        tanks: Vec::new(),
        created_at: time(),
        updated_at: None,
    };

    FARM_STORAGE.with(|s| s.borrow_mut().insert(id, farm.clone()));

    Ok(farm)
}

// update a farm
#[ic_cdk::update]
fn update_farm(farm_id: u64, farm_payload: FarmPayload) -> Result<Farm,Error> {
    // payload Validation all in one line
    if farm_payload.farm_name.is_empty() || farm_payload.location.is_empty() {
        return Err(Error::NotFound {
            msg: format!("Invalid Farm Payload. Should not be empty or zero"),
        });
    }   
    match FARM_STORAGE.with(|service| service.borrow().get(&farm_id)) {
        Some(mut farm) => {
            farm.farm_name = farm_payload.farm_name;
            farm.location = farm_payload.location;
            farm.updated_at = Some(time());
            FARM_STORAGE.with(|m| m.borrow_mut().insert(farm_id, farm.clone()));
            Ok(farm)
        }
        None => Err(Error::NotFound {
            msg: format!("farm with id={} not found", farm_id),
        }),
    }

}

// insert tank into farm
#[ic_cdk::update]
fn insert_tank_into_farm(farm_id: u64, tank_id: u64) -> Result<Farm,Error> {
    match FARM_STORAGE.with(|service| service.borrow().get(&farm_id)) {
        Some(mut farm) => {
            match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
                Some(tank) => {
                    farm.tanks.push(tank.clone());
                    farm.updated_at = Some(time());
                    FARM_STORAGE.with(|m| m.borrow_mut().insert(farm_id, farm.clone()));
                    Ok(farm)
                }
                None => Err(Error::NotFound {
                    msg: format!("tank with id={} not found", tank_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("farm with id={} not found", farm_id),
        }),
    }
}

//remove tank from farm
#[ic_cdk::update]
fn remove_tank_from_farm(farm_id: u64, tank_id: u64) -> Result<Farm,Error> {
    match FARM_STORAGE.with(|service| service.borrow().get(&farm_id)) {
        Some(mut farm) => {
            match TANK_STORAGE.with(|service| service.borrow().get(&tank_id)) {
                Some(tank) => {
                    let index = farm.tanks.iter().position(|x| x.id == tank.id).unwrap();
                    farm.tanks.remove(index);
                    farm.updated_at = Some(time());
                    FARM_STORAGE.with(|m| m.borrow_mut().insert(farm_id, farm.clone()));
                    Ok(farm)
                }
                None => Err(Error::NotFound {
                    msg: format!("tank with id={} not found", tank_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("farm with id={} not found", farm_id),
        }),
    }
}



// get a farm by id
#[ic_cdk::query]
fn get_farm(farm_id: u64) -> Result<Farm,Error> {
    FARM_STORAGE.with(|service| {
        service
            .borrow_mut()
            .get(&farm_id)
            .ok_or(Error::NotFound {
                msg: format!("Farm with id={} not found", farm_id),
            })
    })
}

// get all farms
#[ic_cdk::query]
fn get_all_farms() -> Vec<Farm> {
    FARM_STORAGE.with(|service| {
        service
            .borrow_mut()
            .iter()
            .map(|(_, farm)| farm.clone())
            .collect()
    })
}

// delete a farm by id
#[ic_cdk::update]
fn delete_farm(farm_id: u64) -> Result<(),Error> {
    match FARM_STORAGE.with(|service| service.borrow_mut().remove(&farm_id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("farm with id={} not found", farm_id),
        }),
    }
}

// Error Handling

#[derive(candid::CandidType, Deserialize, Serialize)]
enum  Error {
    NotFound { msg: String },
}

// Export the candid interface
ic_cdk::export_candid!();
