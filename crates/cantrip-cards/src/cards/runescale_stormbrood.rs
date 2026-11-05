use cantrip_api::*;
use cantrip_mechanics::triggers::{noncreature_spell_you_cast, subtype_spell_you_cast};
use cantrip_mechanics::{counter_target, effects, lands, targets};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell or a Dragon spell" (CR 603.2): one trigger for a spell that is both, judged on the
/// spell as it is on the stack (CR 601.2i) - an Omen cast as its Omen is a noncreature spell (CR 720.3b) - with "Dragon" read through
/// the words of this card's text (CR 612).
fn noncreature_or_dragon_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    noncreature_spell_you_cast(q, me, event).or_else(|| subtype_spell_you_cast::<{ lands::sub(Subtype::Dragon) }>(q, me, event))
}

/// "Counter target spell with mana value 2 or less" (CR 701.6); a spell whose target became illegal is not resolved (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen instant for {1}{U}; as it resolves it is shuffled into its owner's library
/// (CR 720.3d, the core's).
pub static CHILLING_SCREECH: CardDef = CardDef {
    spell: spell(&[target(targets::spell_with_mana_value_at_most::<2>)], counter),
    ..oracle::CHILLING_SCREECH
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, noncreature_or_dragon_spell_you_cast),
        effects::this_gets::<2, 0>,
        "Whenever you cast a noncreature spell or a Dragon spell, this creature gets +2/+0 until end of turn.",
    )],
    omen: Some(&CHILLING_SCREECH),
    ..oracle::RUNESCALE_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&CHILLING_SCREECH];
