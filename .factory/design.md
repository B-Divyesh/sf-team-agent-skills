# Team Skills Registry visual thesis

## Direction — paper-cut release desk

This is a paper-cut diorama of controlled software delivery: skill packets travel
from a review desk, through a stamped approval gate, and into separated repository
drawers. Layered paper is a useful visual metaphor for instructions that must be
reviewed, versioned, and deliberately released. It avoids the weightless,
interchangeable SaaS look while keeping operational details easy to scan.

## Palette

| Token | Value | Use |
| --- | --- | --- |
| `ink` | `#182B3A` | main text, deep paper shadow |
| `paper` | `#F7F0DE` | warm background |
| `paper-deep` | `#E9DAB9` | secondary paper planes |
| `forest` | `#176B5D` | primary action and approved state |
| `forest-dark` | `#0D4D43` | action hover and high-contrast text |
| `coral` | `#A84131` | risk and blocked state; darkened for text contrast |
| `sun` | `#E4A92B` | review / pending state |
| `night` | `#10212C` | dark treatment and footer |

The light workspace is the default; the dark footer and deep shadows make the
cut-paper layers feel physically nested. All text/action combinations meet 4.5:1.

## Type and rhythm

The display face is Georgia, a practical serif that gives release notes the
authority of a field manual. The UI face is the self-hosted-safe system sans stack
(`ui-sans-serif`, `Segoe UI`, `Arial`) for dense operational labels. The scale is
1.25 with body at 16px / 1.55. Spacing uses an 8px base, with 16px table gaps,
24px component gaps, and 48–80px section gaps.

## Interaction and motion

Buttons press down into their paper layer. Selecting a skill slides its receipt
from the same layer; approval rings change a physical stamp. Motion stays at
180–240ms and uses transform/opacity only. With reduced motion, all changes are
instant and the diorama has no drift.

Recovery keys and repository-native exports use bordered paper receipts within
the same desk metaphor. They add operational controls without introducing a
generic dashboard card style.

## Assets and provenance

The hero art is an original generated paper-cut release desk: repository drawers,
a stamped skill packet, and a receipt strip, with no readable text or logos.
Prompt sheet: warm editorial paper craft, top-down shallow diorama, deckled paper,
forest green, coral red, mustard, deep navy shadows, soft studio light, 50mm lens;
negative: text, watermark, logo, people, brand marks, UI screenshot.

Generated with Azure AI Foundry through `/opt/fleet/lib/gen-image.sh` on
2026-08-28. It is original product artwork. The exported WebP is used for the
hero and social card; its source prompt is recorded beside the image.
