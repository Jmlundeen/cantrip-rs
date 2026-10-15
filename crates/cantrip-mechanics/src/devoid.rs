//! Devoid (CR 702.114a): "This object is colorless." A characteristic-defining ability, so it holds in every zone.
//! Sowing Mycospawn prints it.
//!
//! A card with devoid gives `COLORS` as its `colors`.

use cantrip_api::*;

/// Colorless, whatever its mana cost.
pub const COLORS: Option<ColorSet> = Some(ColorSet::NONE);
