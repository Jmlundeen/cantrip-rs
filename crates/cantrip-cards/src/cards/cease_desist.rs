use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Cease: "Exile up to two target cards from a single graveyard. Target player gains 2 life and draws a card." The cards
/// still in that graveyard are exiled together (CR 608.2b: a target that became illegal is left alone and the rest of
/// the spell still happens), then the player, if still a legal target, gains 2 life and draws.
fn cease(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> = (0..2).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.exile_all(&cards);
    if g.target_legal(item, 2)
        && let Target::Player(player) = item.targets[2]
    {
        g.gain_life(player, 2);
        g.draw(player, 1);
    }
    Ok(())
}

/// Desist: "Destroy all artifacts and enchantments." Every one on the battlefield goes at once (CR 701.8a, 101.4); an
/// artifact creature or enchantment creature too, and indestructible or regeneration is the verb's business.
fn desist(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Artifact) || c.is(CardType::Enchantment)
        })
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CEASE: CardDef = CardDef {
    spell: spell(
        &[
            targets::CARD_IN_A_GRAVEYARD.optional().labelled("First card to exile (up to two, from a single graveyard)"),
            targets::ANOTHER_CARD_IN_THE_SAME_GRAVEYARD.optional().labelled("Second card to exile, from the same graveyard"),
            targets::PLAYER.labelled("Player to gain 2 life and draw a card"),
        ],
        cease,
    ),
    ..oracle::CEASE
};

pub static DESIST: CardDef = CardDef { spell: spell(&[], desist), ..oracle::DESIST };

pub static CARD: CardDef = CardDef { split: Some((&CEASE, &DESIST)), ..oracle::CEASE_DESIST };

pub static EXTRA: &[&CardDef] = &[&CEASE, &DESIST];
