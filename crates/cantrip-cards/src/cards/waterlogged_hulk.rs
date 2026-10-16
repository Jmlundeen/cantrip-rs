use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::targets::is_permanent_card;
use cantrip_mechanics::craft;
use cantrip_oracle as oracle;

/// "{T}: Mill a card." (CR 701.17)
fn mill_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 1);
    Ok(())
}

/// "Craft with Island": an Island you control or an Island card from your graveyard (CR 702.167b), the basic land type read
/// through the Hulk's words (CR 612).
fn island(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Island))
}

static ISLAND: Materials = craft::materials(1, false, island, "Island");

/// "Descend 8 - This Vehicle can't be blocked as long as there are eight or more permanent cards in your graveyard" (CR
/// 207.2c: descend is an ability word; CR 110.4b): counted in its controller's graveyard whenever its characteristics are.
fn unblockable_with_eight(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let s = q.state();
    let permanents = s.zone(Zone::graveyard(s.obj(source).controller)).iter().filter(|&&c| is_permanent_card(&q.chars(c))).count();
    if permanents >= 8 {
        chars.keywords = chars.keywords.with(Keyword::CantBeBlocked);
    }
}

pub static BACK: CardDef = CardDef {
    abilities: &[crew(1, "Crew 1")],
    statics: &[static_ability(Layer::Ability, itself, unblockable_with_eight, "Descend 8 \u{2014} This Vehicle can't be blocked as long as there are eight or more permanent cards in your graveyard.")],
    ..oracle::WATERTIGHT_GONDOLA
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, mill_one, "{T}: Mill a card."), craft::ability(mana("{3}{U}"), &ISLAND, "Craft with Island {3}{U}")],
    back: Some(&BACK),
    ..oracle::WATERLOGGED_HULK
};
