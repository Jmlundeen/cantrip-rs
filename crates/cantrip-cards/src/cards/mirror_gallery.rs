use cantrip_api::*;
use cantrip_oracle as oracle;

/// "The 'legend rule' doesn't apply." A static ability (CR 604.1) that changes the state-based action of CR 704.5j for
/// every legendary permanent, whoever controls it: two with the same name under one player both stay. It applies only
/// while Mirror Gallery is on the battlefield with its abilities; once it leaves, the rule applies the next time
/// state-based actions are checked, each player whose legends share a name choosing the one to keep (the ruling).
fn every_legend(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

pub static CARD: CardDef = CardDef { legend_rule_exempt: &[every_legend], ..oracle::MIRROR_GALLERY };
