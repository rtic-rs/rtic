use syntax::Statics;
use util::Ceilings;

pub fn resources(resources: &Statics, ceilings: &Ceilings) {
    for resource in resources.keys() {
        assert!(
            ceilings.get(&resource).is_some(),
            "resource {} is unused",
            resource
        );
    }
}
