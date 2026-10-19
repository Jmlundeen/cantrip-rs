use cantrip_api::*;
use cantrip_mechanics::{evoke, spent, targets, triggers::THIS_ENTERS};
use cantrip_mechanics::{effects, exile_target};
use cantrip_oracle as oracle;

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[evoke::way(Cost::mana(mana("{G/U}{G/U}")), "Evoke {G/U}{G/U}")],
    abilities: &[
        evoke::sacrifice_trigger("When Wistfulness enters, if its evoke cost was paid, its controller sacrifices it."),
        triggered(THIS_ENTERS, exile_it, "When this creature enters, if {G}{G} was spent to cast it, exile target artifact or enchantment an opponent controls.")
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT_AN_OPPONENT_CONTROLS])
            .condition(spent::two::<{ Mana::G as u8 }>),
        triggered(THIS_ENTERS, effects::draw_two_then_discard, "When this creature enters, if {U}{U} was spent to cast it, draw two cards, then discard a card.")
            .condition(spent::two::<{ Mana::U as u8 }>),
    ],
    ..oracle::WISTFULNESS
};
