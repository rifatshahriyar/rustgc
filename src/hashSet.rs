lazy_static! {
    pub static ref myHashSet : RwLock<HashSet<Address>> = {
        let mut ret :HashSet<Address> = HashSet::new();
        RwLock::new(ret)
    };
}