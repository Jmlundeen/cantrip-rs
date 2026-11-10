use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

fn pay_to_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_untap_self(g, item, cost("{U}{U}{U}"))
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            pay_to_untap,
            "At the beginning of your upkeep, you may pay {U}{U}{U}. If you do, untap this creature.",
        ),
        state_triggered(
            lands::you_control_no_islands,
            effects::sacrifice_self,
            "When you control no Islands, sacrifice this creature.",
        ),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::ISLAND_FISH_JASCONIUS
};
