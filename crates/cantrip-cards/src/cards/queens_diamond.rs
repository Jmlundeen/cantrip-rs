use cantrip_api::*;
use cantrip_mechanics::{mana::one_of_the_chosen_color_per_charge_counter, statics, station};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying, vigilance, haste";

/// "Flying, vigilance and haste" are granted by the 8+ striation, not printed (CR 721.2a): in one layer 6 part that
/// continues the layer 4 type change (CR 613.6).
fn have_flying_vigilance_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars
        .keywords
        .with(Keyword::Flying)
        .with(Keyword::Vigilance)
        .with(Keyword::Haste);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_any_color(1)
                .by(one_of_the_chosen_color_per_charge_counter)
                .only_for(ManaRestriction::YourCommander),
            "{T}: Choose a color. Add one mana of that color for each charge counter on Queen's Diamond. Spend this mana only to cast your commander.",
        ),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)",
        ),
    ],
    statics: &[
        // "{8+}[abilities][P/T]" (CR 721.2a-b): while it has eight or more charge counters it is an artifact creature with
        // base power and toughness 5/5 in addition to its other types, and has the striation's static ability.
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, statics::itself, have_flying_vigilance_and_haste, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<5, 5>, EIGHT_PLUS).continues(),
    ],
    ..oracle::QUEENS_DIAMOND
};
