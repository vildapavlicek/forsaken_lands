# Palette's Journal

## 2024-10-26 - Missing Visual Feedback Pattern
**Learning:** The application currently relies on static colors for buttons, leading to a lack of tactile feel. Bevy's `Interaction` component makes it easy to add this, but it must be manually implemented for each button type or through a generic system.
**Action:** Implement a generic or specific system to handle `Interaction` changes for all buttons to provide immediate visual feedback.
