use crate::box_model::{Dimensions, Rect};
use breeze_css::style::{BoxSizing, ComputedStyle, FlexDirection, JustifyContent, Position};
use breeze_html::dom::NodeRef;

#[derive(Debug, Clone, PartialEq)]
pub enum BoxType {
    Block,
    Inline,
    Flex,
    Grid,
    Text(String),
}

pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub style: ComputedStyle,
    pub node: Option<NodeRef>,
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    pub fn new(box_type: BoxType, style: ComputedStyle, node: Option<NodeRef>) -> Self {
        Self {
            dimensions: Dimensions::new(),
            box_type,
            style,
            node,
            children: Vec::new(),
        }
    }

    pub fn layout(&mut self, containing_block: &Rect) {
        match self.box_type {
            BoxType::Block => self.layout_block(containing_block),
            BoxType::Flex => self.layout_flex(containing_block),
            BoxType::Grid => self.layout_grid(containing_block),
            BoxType::Inline | BoxType::Text(_) => self.layout_inline(containing_block),
        }

        if self.style.position == Position::Relative {
            if let Some(top) = self.style.top {
                self.dimensions.content.y += top;
            }
            if let Some(left) = self.style.left {
                self.dimensions.content.x += left;
            }
        }
    }

    fn calculate_block_width(&mut self, containing_block: &Rect) {
        let style = &self.style;

        self.dimensions.margin.left = style.margin_left;
        self.dimensions.margin.right = style.margin_right;
        self.dimensions.margin.top = style.margin_top;
        self.dimensions.margin.bottom = style.margin_bottom;

        self.dimensions.padding.left = style.padding_left;
        self.dimensions.padding.right = style.padding_right;
        self.dimensions.padding.top = style.padding_top;
        self.dimensions.padding.bottom = style.padding_bottom;

        self.dimensions.border.left = style.border_left_width;
        self.dimensions.border.right = style.border_right_width;
        self.dimensions.border.top = style.border_top_width;
        self.dimensions.border.bottom = style.border_bottom_width;

        let total_spacing = self.dimensions.margin.left
            + self.dimensions.margin.right
            + self.dimensions.padding.left
            + self.dimensions.padding.right
            + self.dimensions.border.left
            + self.dimensions.border.right;

        if let Some(w) = style.width {
            if style.box_sizing == BoxSizing::BorderBox {
                let border_padding = self.dimensions.padding.left
                    + self.dimensions.padding.right
                    + self.dimensions.border.left
                    + self.dimensions.border.right;
                self.dimensions.content.width = (w - border_padding).max(0.0);
            } else {
                self.dimensions.content.width = w;
            }
        } else {
            self.dimensions.content.width = (containing_block.width - total_spacing).max(0.0);
        }

        self.dimensions.content.x = containing_block.x
            + self.dimensions.margin.left
            + self.dimensions.border.left
            + self.dimensions.padding.left;
    }

    fn layout_block(&mut self, containing_block: &Rect) {
        self.calculate_block_width(containing_block);

        self.dimensions.content.y = containing_block.y
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;

        let mut cursor_y = self.dimensions.content.y;
        let mut cursor_x = self.dimensions.content.x;
        let mut line_height: f32 = 0.0;

        let has_inline_children = self.children.iter().any(|c| matches!(c.box_type, BoxType::Inline | BoxType::Text(_)));

        if has_inline_children {
            for child in &mut self.children {
                let child_rect = Rect::new(
                    cursor_x,
                    cursor_y,
                    self.dimensions.content.width - (cursor_x - self.dimensions.content.x),
                    0.0,
                );
                child.layout(&child_rect);

                let item_width = child.dimensions.margin_box().width;
                let item_height = child.dimensions.margin_box().height;

                if cursor_x + item_width > self.dimensions.content.x + self.dimensions.content.width && cursor_x > self.dimensions.content.x {
                    cursor_y += line_height;
                    cursor_x = self.dimensions.content.x;
                    line_height = item_height;

                    child.dimensions.content.x = cursor_x;
                    child.dimensions.content.y = cursor_y;
                    cursor_x += item_width;
                } else {
                    line_height = line_height.max(item_height);
                    cursor_x += item_width;
                }
            }
            cursor_y += line_height;
        } else {
            for child in &mut self.children {
                let child_containing = Rect::new(
                    self.dimensions.content.x,
                    cursor_y,
                    self.dimensions.content.width,
                    0.0,
                );
                child.layout(&child_containing);
                cursor_y += child.dimensions.margin_box().height;
            }
        }

        if let Some(h) = self.style.height {
            self.dimensions.content.height = h;
        } else {
            self.dimensions.content.height = cursor_y - self.dimensions.content.y;
        }
    }

    fn layout_flex(&mut self, containing_block: &Rect) {
        self.calculate_block_width(containing_block);
        self.dimensions.content.y = containing_block.y
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;

        let gap = self.style.gap;
        let is_row = self.style.flex_direction == FlexDirection::Row;

        if is_row {
            let mut total_fixed_width = 0.0;
            let mut total_grow = 0.0;
            let n = self.children.len();

            for (i, child) in self.children.iter_mut().enumerate() {
                let initial_w = child.style.width.unwrap_or(80.0);
                total_fixed_width += initial_w;
                if i > 0 {
                    total_fixed_width += gap;
                }
                total_grow += child.style.flex_grow;
            }

            let remaining_space = (self.dimensions.content.width - total_fixed_width).max(0.0);

            let mut cursor_x = self.dimensions.content.x;
            if total_grow == 0.0 {
                match self.style.justify_content {
                    JustifyContent::Center => {
                        cursor_x += remaining_space / 2.0;
                    }
                    JustifyContent::FlexEnd => {
                        cursor_x += remaining_space;
                    }
                    _ => {}
                }
            }

            let mut max_child_height = 0.0f32;

            for (_i, child) in self.children.iter_mut().enumerate() {
                let mut child_w = child.style.width.unwrap_or(80.0);
                if total_grow > 0.0 {
                    child_w += (child.style.flex_grow / total_grow) * remaining_space;
                }

                let child_rect = Rect::new(cursor_x, self.dimensions.content.y, child_w, self.dimensions.content.height);
                child.dimensions.content.width = child_w;
                child.layout(&child_rect);

                let h = child.dimensions.margin_box().height;
                max_child_height = max_child_height.max(h);

                cursor_x += child.dimensions.margin_box().width + gap;
            }

            self.dimensions.content.height = self.style.height.unwrap_or(max_child_height);
        } else {
            let mut cursor_y = self.dimensions.content.y;
            for child in &mut self.children {
                let child_rect = Rect::new(
                    self.dimensions.content.x,
                    cursor_y,
                    self.dimensions.content.width,
                    0.0,
                );
                child.layout(&child_rect);
                cursor_y += child.dimensions.margin_box().height + gap;
            }
            self.dimensions.content.height = self.style.height.unwrap_or(cursor_y - self.dimensions.content.y);
        }
    }

    fn layout_grid(&mut self, containing_block: &Rect) {
        self.calculate_block_width(containing_block);
        self.dimensions.content.y = containing_block.y
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;

        let num_cols = if !self.style.grid_template_columns.is_empty() {
            self.style.grid_template_columns.len()
        } else {
            3
        };

        let gap = self.style.grid_gap;
        let total_gap = gap * (num_cols - 1) as f32;
        let col_width = ((self.dimensions.content.width - total_gap) / num_cols as f32).max(10.0);

        let mut col = 0;
        let mut row_height = 0.0f32;
        let mut total_height = 0.0f32;

        for child in &mut self.children {
            let child_x = self.dimensions.content.x + col as f32 * (col_width + gap);
            let child_y = self.dimensions.content.y + total_height;

            let child_rect = Rect::new(child_x, child_y, col_width, 0.0);
            child.dimensions.content.width = col_width;
            child.layout(&child_rect);

            row_height = row_height.max(child.dimensions.margin_box().height);
            col += 1;
            if col >= num_cols {
                col = 0;
                total_height += row_height + gap;
                row_height = 0.0;
            }
        }
        if col > 0 {
            total_height += row_height;
        }

        self.dimensions.content.height = self.style.height.unwrap_or(total_height);
    }

    fn layout_inline(&mut self, containing_block: &Rect) {
        self.dimensions.content.x = containing_block.x;
        self.dimensions.content.y = containing_block.y;

        match &self.box_type {
            BoxType::Text(t) => {
                let char_width = self.style.font_size * 0.55;
                let text_len = t.len() as f32;
                self.dimensions.content.width = (char_width * text_len).min(containing_block.width);
                self.dimensions.content.height = self.style.font_size * 1.25;
            }
            _ => {
                let mut cursor_x = self.dimensions.content.x;
                let mut max_h = self.style.font_size * 1.2;
                for child in &mut self.children {
                    let child_rect = Rect::new(cursor_x, self.dimensions.content.y, containing_block.width - (cursor_x - self.dimensions.content.x), 0.0);
                    child.layout(&child_rect);
                    cursor_x += child.dimensions.margin_box().width;
                    max_h = max_h.max(child.dimensions.margin_box().height);
                }
                self.dimensions.content.width = cursor_x - self.dimensions.content.x;
                self.dimensions.content.height = max_h;
            }
        }
    }
}
