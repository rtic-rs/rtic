use syntax::Resources;
use util::{Ceiling, Ceilings};

pub fn resources(resources: &Resources, ceilings: &Ceilings) {
    for resource in resources.keys() {
        if let Some(ceiling) = ceilings.get(&resource) {
            assert_ne!(
                *ceiling,
                Ceiling::Owned,
                "{} should be local data",
                resource
            );
        } else {
            panic!("resource {} is unused", resource)
        }
    }
}
