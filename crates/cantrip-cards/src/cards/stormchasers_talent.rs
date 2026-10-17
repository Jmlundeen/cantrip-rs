use cantrip_api::*;
use cantrip_mechanics::{class, targets, tokens};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn otter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OTTER, 1)?;
    Ok(())
}

/// "Whenever you cast an instant or sorcery spell", at level 3.
fn cast_at_level_three(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if class::level(q, me.source) < 3 {
        return None;
    }
    cantrip_mechanics::triggers::you_cast_instant_or_sorcery(q, me, event)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        class::level_two("{3}{U}", "{3}{U}: Level 2"),
        class::level_three("{5}{U}", "{5}{U}: Level 3"),
        triggered(THIS_ENTERS, otter, "When this Class enters, create a 1/1 blue and red Otter creature token with prowess."),
        triggered(class::BECOMES_LEVEL_TWO, cantrip_mechanics::effects::return_first_target_to_hand, "When this Class becomes level 2, return target instant or sorcery card from your graveyard to your hand.")
            .targets(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD]),
        triggered(when_asked(EventMask::SPELL_CAST, cast_at_level_three), otter, "Whenever you cast an instant or sorcery spell, create a 1/1 blue and red Otter creature token with prowess."),
    ],
    ..oracle::STORMCHASERS_TALENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::OTTER];
