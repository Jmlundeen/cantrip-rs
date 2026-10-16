use cantrip_api::*;
use cantrip_mechanics::tokens::{self, FISH};
use cantrip_oracle as oracle;

/// "Sacrifice a token" (CR 701.21): a token its controller controls, whatever it is.
fn token_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.obj(id).token && q.controller(id) == asker.controller
}

/// "Draw a card."
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "Create a 1/1 blue Fish creature token."
fn create_fish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &FISH, 1)?;
    Ok(())
}

/// "Create a Treasure token."
fn create_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}, {T}").and(sacrifice(1, token_you_control)), draw_a_card, "{2}, {T}, Sacrifice a token: Draw a card."),
        ability(cost("{3}, {T}").and(life(1)), create_fish, "{3}, {T}, Pay 1 life: Create a 1/1 blue Fish creature token."),
        ability(cost("{4}, {T}"), create_treasure, "{4}, {T}: Create a Treasure token."),
    ],
    ..oracle::FOUNTAINPORT
};

pub static EXTRA: &[&CardDef] = &[&FISH, &tokens::TREASURE];
