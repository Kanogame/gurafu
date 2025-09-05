```c
// pan_zoom_with_scale_commented.c
// Pan & zoom camera with scale stored in the view.
// - Language: C
// - Compile: gcc -O2 -o pan_zoom_with_scale_commented pan_zoom_with_scale_commented.c
// - Run: ./pan_zoom_with_scale_commented
//
// Terminology:
// - World (aka "canvas" or "content"): the coordinate space where your shapes/live data live.
//   Example: a circle at world (400,300) with radius 50 means "400 units right, 300 units down"
//   inside the logical canvas. Units are arbitrary (pixels, meters, etc.) but consistent.
// - Screen (aka "viewport" or "display"): the pixel rectangle on the user's monitor/window
//   where the world is rendered. Screen coords are in pixels with origin at top-left (0,0).
// - ViewRect: represents the camera into the world. It defines which world rectangle is
//   currently visible on the screen by storing the world-space origin (top-left) and a
//   uniform scale (world units per screen pixel).
// - Viewport: describes the size of the screen area (width and height in screen pixels).
//
// Concepts:
// - scale = world units per screen pixel. Larger scale => objects appear larger in world units
//   per pixel (i.e., zoomed in means smaller scale value here: scale decreases when zooming in).
// - view_width_world = scale * viewport.w (world-space width that fits the viewport).
// - screen_to_world/world_to_screen convert coordinates between spaces.
// - Zoom is centered at a screen point so the world point under the cursor remains fixed.

#include <stdio.h>
#include <stdlib.h>
#include <math.h>

/* ViewRect: camera describing which part of the world is visible.
   - x,y: world-space coordinates of the top-left of the visible rectangle.
   - scale: world units per screen pixel (uniform in x and y).
     Example:
       scale = 1.0 -> 1 world unit maps to 1 screen pixel
       scale = 0.5 -> 0.5 world units per pixel (zoomed in: fewer world units fit on screen)
       scale = 2.0 -> 2 world units per pixel (zoomed out: more world units fit on screen)
*/
typedef struct {
    double x, y;      // origin (top-left) in world coordinates
    double scale;     // world units per screen pixel (must be > 0)
} ViewRect;

/* Viewport: the screen rectangle (in pixels) where the world is drawn.
   - w,h: dimensions in screen pixels.
*/
typedef struct {
    double w, h; // viewport size in screen pixels
} Viewport;

/* ZoomLimits: allowed range for scale (world units per pixel).
   - min_scale: the smallest allowed scale value (cannot zoom in past this).
   - max_scale: the largest allowed scale value (cannot zoom out past this).
*/
typedef struct {
    double min_scale;
    double max_scale;
} ZoomLimits;

/* Derived size of the view in world units given the current scale and viewport size. */
static inline double view_width_world(const ViewRect *v, const Viewport *vp) { return v->scale * vp->w; }
static inline double view_height_world(const ViewRect *v, const Viewport *vp) { return v->scale * vp->h; }

/* Initialize the view:
   - origin_x, origin_y: initial top-left in world coords
   - initial_scale: initial world units per pixel (must be > 0, falls back to 1.0 otherwise)
*/
void view_init(ViewRect *v, double origin_x, double origin_y, double initial_scale) {
    v->x = origin_x;
    v->y = origin_y;
    v->scale = (initial_scale > 0.0) ? initial_scale : 1.0;
}

/* Convert screen (pixel) coordinates to world coordinates.
   - sx, sy: screen coords (pixels) relative to viewport top-left.
   - wx, wy: outputs in world coordinates.
   Formula: world = view_origin + screen * scale
*/
void screen_to_world(const ViewRect *v, const Viewport *vp, double sx, double sy, double *wx, double *wy) {
    (void)vp; // vp isn't needed because scale is stored in ViewRect, but kept for API symmetry
    *wx = v->x + sx * v->scale;
    *wy = v->y + sy * v->scale;
}

/* Convert world coordinates to screen (pixel) coordinates.
   - wx, wy: world coordinates
   - sx, sy: outputs in screen pixels relative to viewport top-left.
   Formula: screen = (world - view_origin) / scale
*/
void world_to_screen(const ViewRect *v, const Viewport *vp, double wx, double wy, double *sx, double *sy) {
    (void)vp;
    *sx = (wx - v->x) / v->scale;
    *sy = (wy - v->y) / v->scale;
}

/* Zoom around a screen point (cursor_sx, cursor_sy).
   - zoom_factor > 1.0 => zoom in (view shows fewer world units => scale decreases).
   - zoom_factor < 1.0 => zoom out (view shows more world units => scale increases).
   The function preserves the world point under the cursor so that it does not jump.
   Limits are enforced if 'limits' is non-NULL.
*/
void view_zoom_at(ViewRect *v, const Viewport *vp, double cursor_sx, double cursor_sy, double zoom_factor, const ZoomLimits *limits) {
    if (zoom_factor <= 0.0) return;

    // World coords currently under the cursor
    double wx = v->x + cursor_sx * v->scale;
    double wy = v->y + cursor_sy * v->scale;

    // new scale: dividing by zoom_factor because larger zoom_factor -> zoom in -> fewer world units per pixel
    double new_scale = v->scale / zoom_factor;

    // Enforce limits (if provided)
    if (limits) {
        if (new_scale < limits->min_scale) new_scale = limits->min_scale;
        if (new_scale > limits->max_scale) new_scale = limits->max_scale;
    }

    // Recompute origin so that (wx,wy) maps to (cursor_sx,cursor_sy) under the new scale:
    // v->x = wx - cursor_sx * new_scale
    v->x = wx - cursor_sx * new_scale;
    v->y = wy - cursor_sy * new_scale;
    v->scale = new_scale;
}

/* Pan the view by a screen-space delta.
   - dx_pixels, dy_pixels: movement in screen pixels (positive means move right/down on screen).
   Pan changes the world origin by screen_delta * scale (because scale is world units per pixel).
   Example: to implement drag where mouse delta is (mx, my),
     call view_pan_by_screen_delta(view, -mx, -my) if you want content to follow the cursor.
*/
void view_pan_by_screen_delta(ViewRect *v, double dx_pixels, double dy_pixels) {
    v->x += dx_pixels * v->scale;
    v->y += dy_pixels * v->scale;
}

/* Clamp the view so it does not show areas outside the world bounds [0..world_w] x [0..world_h].
   If the view is larger than the world, center it.
*/
void clamp_view_to_world(ViewRect *v, const Viewport *vp, double world_w, double world_h) {
    double vw = view_width_world(v, vp);
    double vh = view_height_world(v, vp);

    if (vw >= world_w) {
        // If the visible world width >= total world width, center horizontally
        v->x = (world_w - vw) * 0.5;
    } else {
        if (v->x < 0.0) v->x = 0.0;
        if (v->x + vw > world_w) v->x = world_w - vw;
    }

    if (vh >= world_h) {
        // center vertically
        v->y = (world_h - vh) * 0.5;
    } else {
        if (v->y < 0.0) v->y = 0.0;
        if (v->y + vh > world_h) v->y = world_h - vh;
    }
}

/* draw_circle_world:
   - world_cx, world_cy, world_r: circle specified in world coordinates (canvas units).
   - draw_circle_screen: callback that draws a circle in screen coordinates: (sx,sy) in pixels, radius in pixels.
   Conversion:
     screen_center = (world_center - view_origin) / scale
     screen_radius = world_radius / scale
   Note: if scale is non-uniform (not in this design), you'd map radius separately for x/y.
*/
void draw_circle_world(const ViewRect *v, const Viewport *vp, double world_cx, double world_cy, double world_r,
                       void (*draw_circle_screen)(double, double, double))
{
    double sx = (world_cx - v->x) / v->scale;      // screen x in pixels
    double sy = (world_cy - v->y) / v->scale;      // screen y in pixels
    double screen_r = world_r / v->scale;          // radius in pixels
    draw_circle_screen(sx, sy, screen_r);
}

/* Example fake screen draw function (prints values). Replace with your renderer. */
static void fake_draw_circle(double sx, double sy, double r) {
    printf("Draw circle at screen (%.2f, %.2f) radius %.2f px\n", sx, sy, r);
}

/* Example usage: demonstrates initialization, zoom, pan, and drawing. */
int main(void) {
    Viewport vp = {800.0, 600.0};          // viewport size in screen pixels
    ViewRect view;
    view_init(&view, 0.0, 0.0, 1.0);       // start with origin (0,0) and scale=1 world/pixel

    ZoomLimits zlim = { 0.1, 10.0 };       // allowed scale range (world units per pixel)

    printf("Initial view origin=(%.2f,%.2f) scale=%.3f (world units per pixel)\n", view.x, view.y, view.scale);

    // Draw a circle specified in world coordinates
    draw_circle_world(&view, &vp, 400.0, 300.0, 50.0, fake_draw_circle);

    // Zoom in 2x at viewport center
    view_zoom_at(&view, &vp, vp.w * 0.5, vp.h * 0.5, 2.0, &zlim);
    printf("After zoom: origin=(%.2f,%.2f) scale=%.3f\n", view.x, view.y, view.scale);
    draw_circle_world(&view, &vp, 400.0, 300.0, 50.0, fake_draw_circle);

    // Pan: simulate dragging the content by 100 px to the right (so pass -100 to move view left)
    view_pan_by_screen_delta(&view, -100.0, 0.0);
    clamp_view_to_world(&view, &vp, 2000.0, 1200.0);
    printf("After pan: origin=(%.2f,%.2f)\n", view.x, view.y);
    draw_circle_world(&view, &vp, 400.0, 300.0, 50.0, fake_draw_circle);

    return 0;
}
```