use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice Arcades Sabboth unless you pay {G}{W}{U}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_arcades_sabboth_unless_pay(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{W}{U}{G}"))
}

/// "Each untapped creature you control gets +0/+2 as long as it's not attacking." (CR 604.1).
fn untapped_creature_control_gets_0_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (!(q.state().obj(affected).tapped) && (*so_far).is(CardType::Creature)
        && asker.controller == q.controller(affected))
        && !(q.state().attackers.iter().any(|a| a.id == affected))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_arcades_sabboth_unless_pay,
            "At the beginning of your upkeep, sacrifice Arcades Sabboth unless you pay {G}{W}{U}.",
        ),
        ability(
            cost("{W}"),
            effects::this_gets::<0, 1>,
            "{W}: Arcades Sabboth gets +0/+1 until end of turn.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            untapped_creature_control_gets_0_affects,
            cantrip_mechanics::statics::plus_pt::<0, 2>,
            "Each untapped creature you control gets +0/+2 as long as it's not attacking.",
        ),
    ],
    ..oracle::ARCADES_SABBOTH
};
