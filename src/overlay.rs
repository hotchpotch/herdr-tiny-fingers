use anyhow::{ensure, Context, Result};
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GEOMETRY_ENV: &str = "HERDR_TINY_FINGERS_GEOMETRY";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayGeometry {
    pub pane_id: String,
    pub area: PaneRect,
    /// Coordinates relative to the tab area, captured before opening the overlay.
    pub pane: PaneRect,
}

impl OverlayGeometry {
    pub fn from_layout(layout: &Value, pane_id: &str) -> Result<Self> {
        let area: PaneRect = serde_json::from_value(layout["area"].clone())?;
        let source = layout["panes"]
            .as_array()
            .context("layout has no panes")?
            .iter()
            .find(|pane| pane["pane_id"].as_str() == Some(pane_id))
            .context("source pane is missing from layout")?;
        let mut pane: PaneRect = if layout["zoomed"].as_bool() == Some(true) {
            area
        } else {
            serde_json::from_value(source["rect"].clone())?
        };
        ensure!(
            area.width > 0 && area.height > 0 && pane.width > 0 && pane.height > 0,
            "empty pane layout"
        );
        ensure!(
            pane.x >= area.x
                && pane.y >= area.y
                && u32::from(pane.x) + u32::from(pane.width)
                    <= u32::from(area.x) + u32::from(area.width)
                && u32::from(pane.y) + u32::from(pane.height)
                    <= u32::from(area.y) + u32::from(area.height),
            "source pane is outside the tab area"
        );
        pane.x -= area.x;
        pane.y -= area.y;
        Ok(Self {
            pane_id: pane_id.to_string(),
            area,
            pane,
        })
    }

    pub fn regions(&self, frame: Rect) -> (Rect, Rect) {
        let content = Rect::new(
            frame.x.saturating_add(self.pane.x),
            frame.y.saturating_add(self.pane.y),
            self.pane.width,
            self.pane.height,
        )
        .intersection(frame);
        // Use unused overlay space for controls so the source pane's last row remains visible.
        let status = if content.bottom() < frame.bottom() {
            Rect::new(frame.x, frame.bottom().saturating_sub(1), frame.width, 1)
        } else if content.y > frame.y {
            Rect::new(frame.x, frame.y, frame.width, 1)
        } else if content.x > frame.x {
            Rect::new(
                frame.x,
                frame.bottom().saturating_sub(1),
                content.x - frame.x,
                1,
            )
        } else if content.right() < frame.right() {
            Rect::new(
                content.right(),
                frame.bottom().saturating_sub(1),
                frame.right() - content.right(),
                1,
            )
        } else {
            Rect::default()
        };
        (content, status.intersection(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn layout(zoomed: bool) -> serde_json::Value {
        json!({"area":{"x":20,"y":2,"width":100,"height":40},"zoomed":zoomed,
            "panes":[{"pane_id":"right","rect":{"x":71,"y":23,"width":49,"height":19}}]})
    }

    #[test]
    fn captures_position_relative_to_tab_before_overlay_opens() {
        let geometry = OverlayGeometry::from_layout(&layout(false), "right").unwrap();
        assert_eq!(
            geometry.pane,
            PaneRect {
                x: 51,
                y: 21,
                width: 49,
                height: 19
            }
        );
        assert_eq!(geometry.area.width, 100);
    }

    #[test]
    fn zoomed_source_uses_whole_tab() {
        let geometry = OverlayGeometry::from_layout(&layout(true), "right").unwrap();
        assert_eq!(
            geometry.pane,
            PaneRect {
                x: 0,
                y: 0,
                width: 100,
                height: 40
            }
        );
    }

    #[test]
    fn rejects_missing_pane_and_out_of_bounds_rectangles() {
        assert!(OverlayGeometry::from_layout(&layout(false), "missing").is_err());
        let mut invalid = layout(false);
        invalid["panes"][0]["rect"]["x"] = json!(19);
        assert!(OverlayGeometry::from_layout(&invalid, "right").is_err());
    }

    #[test]
    fn bottom_right_pane_keeps_last_row_and_status_stays_outside() {
        let geometry = OverlayGeometry::from_layout(&layout(false), "right").unwrap();
        let (content, status) = geometry.regions(Rect::new(0, 0, 100, 40));
        assert_eq!(content, Rect::new(51, 21, 49, 19));
        assert_eq!(status.intersection(content).height, 0);
        assert!(status.width > 0);
    }

    #[test]
    fn side_by_side_pane_places_status_in_other_half() {
        let mut geometry = OverlayGeometry::from_layout(&layout(false), "right").unwrap();
        geometry.pane = PaneRect {
            x: 51,
            y: 0,
            width: 49,
            height: 40,
        };
        let (content, status) = geometry.regions(Rect::new(0, 0, 100, 40));
        assert_eq!(content, Rect::new(51, 0, 49, 40));
        assert_eq!(status, Rect::new(0, 39, 51, 1));
    }

    #[test]
    fn clips_on_smaller_screen_without_reflowing() {
        let geometry = OverlayGeometry::from_layout(&layout(false), "right").unwrap();
        let (content, _) = geometry.regions(Rect::new(0, 0, 80, 30));
        assert_eq!(content, Rect::new(51, 21, 29, 9));
    }
}
