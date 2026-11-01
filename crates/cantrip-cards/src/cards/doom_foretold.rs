use cantrip_api::*;
use cantrip_mechanics::tokens::{self, KNIGHT};
use cantrip_mechanics::triggers::EACH_PLAYERS_UPKEEP;
use cantrip_mechanics::{discard_chosen, effects, sacrifice_one_of};
use cantrip_oracle as oracle;

/// "That player sacrifices a nonland, nontoken permanent of their choice. If that player can't, they discard a card, they lose 2
/// life, you draw a card, you gain 2 life, you create a 2/2 white Knight creature token with vigilance, then you sacrifice this
/// enchantment." (CR 701.21a, 101.2, 608.2c): the player whose upkeep it is must sacrifice if they can; one who sacrificed
/// nothing - no such permanent, or none they can be made to sacrifice - suffers the rest. Each part is done whether or not the
/// one before could be (the rulings): the controller then sacrifices this enchantment unless it has already gone or changed
/// controllers.
fn doom(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    let eligible: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| !g.chars(id).is(CardType::Land) && !g.obj(id).token).collect();
    if !sacrifice_one_of(g, who, item.object, 1, eligible)?.is_empty() {
        return Ok(());
    }
    g.end_sentence();
    discard_chosen(g, who, item.object, 1)?;
    g.lose_life(who, 2);
    effects::draw(g, item, 1)?;
    effects::gain_life(g, item, 2)?;
    tokens::create(g, item.controller, &KNIGHT, 1)?;
    g.end_sentence();
    effects::sacrifice_self(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(EACH_PLAYERS_UPKEEP, doom, "At the beginning of each player's upkeep, that player sacrifices a nonland, nontoken permanent of their choice. If that player can't, they discard a card, they lose 2 life, you draw a card, you gain 2 life, you create a 2/2 white Knight creature token with vigilance, then you sacrifice this enchantment.")],
    ..oracle::DOOM_FORETOLD
};

pub static EXTRA: &[&CardDef] = &[&KNIGHT];
