use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// It deals 1 damage to each opponent, then creates a Blood token.
fn ping_and_blood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 1)).collect();
    g.damage_together(&hits);
    tokens::create_blood(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Voldaren Epicure enters, it deals 1 damage to each opponent. Create a Blood token.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: ping_and_blood,
        ..TriggeredDef::BLANK
    })],
    ..oracle::VOLDAREN_EPICURE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
