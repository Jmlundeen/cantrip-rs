use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::connive;

/// "Whenever a player casts their second spell each turn": any player's, counted from the turn's start, so spells
/// cast before this creature arrived count too.
fn second_spell(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { nth: 2, .. } => Fired::PLAIN,
        _ => None,
    }
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    connive(g, item.object, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a player casts their second spell each turn, this creature connives. (Draw a card, then discard a card. If you discarded a nonland card, put a +1/+1 counter on this creature.)",
        listens: EventMask::SPELL_CAST,
        matches: second_spell,
        effect,
        ..TriggeredDef::BLANK
    })],
    ..oracle::LEDGER_SHREDDER
};
