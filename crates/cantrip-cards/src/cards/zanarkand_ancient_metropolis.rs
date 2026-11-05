use cantrip_api::*;
use cantrip_mechanics::tokens::HERO_1_1_COLORLESS;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a 1/1 colorless Hero creature token. Put a +1/+1 counter on it for each land you control" (CR 111.1, 122.1): the lands
/// counted as the spell resolves, after the token has entered (it is no land), as the layers have made them (a count of none puts no counter, CR 122.1). Every token made (a token doubler makes more) gets them, all at once.
fn create_a_hero_with_a_counter_per_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let made = tokens::create_by_rule(g, you, &HERO_1_1_COLORLESS, 1, false)?;
    let lands = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).count();
    let puts: Vec<_> = made.iter().map(|&token| (token, CounterKind::PlusOne, lands as i32)).collect();
    g.add_counters_together(&puts);
    Ok(())
}

/// The Adventure (CR 715): Create a 1/1 colorless Hero creature token. Put a +1/+1 counter on it for each land you control.
pub static LASTING_FAYTH: CardDef = CardDef { spell: spell(&[], create_a_hero_with_a_counter_per_land), ..oracle::LASTING_FAYTH };

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[mana_ability(TAP, add(Mana::G), "{T}: Add {G}.")],
    adventure: Some(&LASTING_FAYTH),
    ..oracle::ZANARKAND_ANCIENT_METROPOLIS
};

pub static EXTRA: &[&CardDef] = &[&LASTING_FAYTH, &HERO_1_1_COLORLESS];
