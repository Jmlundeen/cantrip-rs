use cantrip_api::select::{part, Choice, SelectFn, Verdict};
use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{library, select, statics, station};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying, lifelink";

/// "A creature or Spacecraft card" (CR 205.3g): a card in a graveyard with the creature type or the Spacecraft subtype.
fn creature_or_spacecraft_card(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let spacecraft = q.words(choice.source).subtype(Subtype::Spacecraft);
    Verdict::each(choice, |card| q.chars(card).is(CardType::Creature) || q.chars(card).has_subtype(spacecraft))
}

const CREATURE_OR_SPACECRAFT_CARD: select::Selection =
    select::Selection::of(select::as_many_as_can(1), &[part(&(creature_or_spacecraft_card as SelectFn), 0, "")]).called("creature or Spacecraft");

/// "Mill three cards, then return a creature or Spacecraft card from your graveyard to your hand."
fn mill_then_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_return_a_card(g, item, 3, CREATURE_OR_SPACECRAFT_CARD)
}

/// The 8+ striation (CR 721.2a): flying and lifelink, in the one layer-6 effect that continues the type change.
fn have_flying_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, mill_then_return, "When this Spacecraft enters, mill three cards, then return a creature or Spacecraft card from your graveyard to your hand."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, statics::itself, have_flying_and_lifelink, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<3, 2>, EIGHT_PLUS).continues(),
    ],
    ..oracle::FELL_GRAVSHIP
};
