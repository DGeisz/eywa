mod encephalon;

pub mod eywa {

    use uuid::Uuid;

    pub fn get_uuid() -> Uuid {
        Uuid::new_v4()
    }
}
