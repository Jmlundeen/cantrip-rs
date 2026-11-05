use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "When this creature dies, create a 2/2 black Zombie Druid creature token" (CR 111.1, 603.10a).
fn create_zombie_druid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ZOMBIE_DRUID, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_DIES, create_zombie_druid, "When this creature dies, create a 2/2 black Zombie Druid creature token."),
        // Renew is an ability word (CR 207.2c): the ability is an ordinary activated ability of the card in the graveyard.
        ability(cost("{B}").and(EXILE_THIS), effects::plus_one_counter_on_target, "Renew \u{2014} {B}, Exile this card from your graveyard: Put a +1/+1 counter on target creature. Activate only as a sorcery.")
            .from(ZoneKind::Graveyard)
            .sorcery_speed()
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::ADORNED_CROCODILE
};

pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE_DRUID];
