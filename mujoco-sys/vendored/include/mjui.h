//--------------------------------//
//  This file is part MuJoCo      //
//  Copyright © 2018, Roboti LLC  //
//--------------------------------//


#pragma once


#define mjMAXUISECT     10      // maximum number of sections
#define mjMAXUIITEM     80      // maximum number of items per section
#define mjMAXUITEXT     500     // maximum number of chars in edittext and other
#define mjMAXUINAME     40      // maximum number of chars in name
#define mjMAXUIMULTI    20      // maximum number of radio/select items in group
#define mjMAXUIEDIT     5       // maximum number of elements in edit list
#define mjMAXUIRECT     15      // maximum number of rectangles


// key codes matching GLFW (user must remap for other frameworks)
#define mjKEY_ESCAPE     256
#define mjKEY_ENTER      257
#define mjKEY_TAB        258
#define mjKEY_BACKSPACE  259
#define mjKEY_INSERT     260
#define mjKEY_DELETE     261
#define mjKEY_RIGHT      262
#define mjKEY_LEFT       263
#define mjKEY_DOWN       264
#define mjKEY_UP         265
#define mjKEY_PAGE_UP    266
#define mjKEY_PAGE_DOWN  267
#define mjKEY_HOME       268
#define mjKEY_END        269
#define mjKEY_F1         290
#define mjKEY_F2         291
#define mjKEY_F3         292
#define mjKEY_F4         293
#define mjKEY_F5         294
#define mjKEY_F6         295
#define mjKEY_F7         296
#define mjKEY_F8         297
#define mjKEY_F9         298
#define mjKEY_F10        299
#define mjKEY_F11        300
#define mjKEY_F12        301


typedef enum _mjtButton         // mouse button
{
    mjBUTTON_NONE = 0,          // no button
    mjBUTTON_LEFT,              // left button
    mjBUTTON_RIGHT,             // right button
    mjBUTTON_MIDDLE             // middle button
} mjtButton;


typedef enum _mjtEvent          // mouse and keyboard event type
{
    mjEVENT_NONE = 0,           // no event
    mjEVENT_MOVE,               // mouse move
    mjEVENT_PRESS,              // mouse button press
    mjEVENT_RELEASE,            // mouse button release
    mjEVENT_SCROLL,             // scroll
    mjEVENT_KEY,                // key press
    mjEVENT_RESIZE              // resize
} mjtEvent;


typedef enum _mjtItem           // UI item type
{
    mjITEM_END = -2,            // end of definition list (not an item)
    mjITEM_SECTION = -1,        // section (not an item)
    mjITEM_SEPARATOR = 0,       // separator
    mjITEM_STATIC,              // static text
    mjITEM_BUTTON,              // button

    // the rest have data pointer
    mjITEM_CHECKINT,            // check box, int value
    mjITEM_CHECKBYTE,           // check box, mjtByte value
    mjITEM_RADIO,               // radio group
    mjITEM_SELECT,              // selection box
    mjITEM_SLIDERINT,           // slider, int value
    mjITEM_SLIDERNUM,           // slider, mjtNum value
    mjITEM_EDITINT,             // editable array, int values
    mjITEM_EDITNUM,             // editable array, mjtNum values
    mjITEM_EDITTXT,             // editable text

    mjNITEM                     // number of item types
} mjtItem;


// predicate function: set enable/disable based on item category
typedef int (*mjfItemEnable)(int category, void* data);


struct _mjuiState               // mouse and keyboard state
{
    // constants set by user
    int nrect;                  // number of rectangles used
    mjrRect rect[mjMAXUIRECT];  // rectangles (index 0: entire window)
    void* userdata;             // pointer to user data (for callbacks)

    // event type
    int type;                   // (type mjtEvent)

    // mouse buttons
    int left;                   // is left button down
    int right;                  // is right button down
    int middle;                 // is middle button down
    int doubleclick;            // is last press a double click
    int button;                 // which button was pressed (mjtButton)
    double buttontime;          // time of last button press

    // mouse position
    double x;                   // x position
    double y;                   // y position
    double dx;                  // x displacement
    double dy;                  // y displacement
    double sx;                  // x scroll
    double sy;                  // y scroll

    // keyboard
    int control;                // is control down
    int shift;                  // is shift down
    int alt;                    // is alt down
    int key;                    // which key was pressed
    double keytime;             // time of last key press

    // rectangle ownership and dragging
    int mouserect;              // which rectangle contains mouse
    int dragrect;               // which rectangle is dragged with mouse
    int dragbutton;             // which button started drag (mjtButton)
};
typedef struct _mjuiState mjuiState;


struct _mjuiThemeSpacing        // UI visualization theme spacing
{
        int total;              // total width
        int scroll;             // scrollbar width
        int label;              // label width
        int section;            // section gap
        int itemside;           // item side gap
        int itemmid;            // item middle gap
        int itemver;            // item vertical gap
        int texthor;            // text horizontal gap
        int textver;            // text vertical gap
        int linescroll;         // number of pixels to scroll
        int samples;            // number of multisamples
};
typedef struct _mjuiThemeSpacing mjuiThemeSpacing;


struct _mjuiThemeColor          // UI visualization theme color
{
    float master[3];            // master background
    float thumb[3];             // scrollbar thumb
    float secttitle[3];         // section title
    float sectfont[3];          // section font
    float sectsymbol[3];        // section symbol
    float sectpane[3];          // section pane
    float shortcut[3];          // shortcut background
    float fontactive[3];        // font active
    float fontinactive[3];      // font inactive
    float decorinactive[3];     // decor inactive
    float decorinactive2[3];    // inactive slider color 2
    float button[3];            // button
    float check[3];             // check
    float radio[3];             // radio
    float select[3];            // select
    float select2[3];           // select pane
    float slider[3];            // slider
    float slider2[3];           // slider color 2
    float edit[3];              // edit
    float edit2[3];             // edit invalid
    float cursor[3];            // edit cursor
};
typedef struct _mjuiThemeColor mjuiThemeColor;


struct _mjuiItem                // UI item
{
    // common properties
    int type;                   // type (mjtItem)
    char name[mjMAXUINAME];     // name
    int state;                  // 0: disable, 1: enable, 2+: use predicate
    void *pdata;                // data pointer (type-specific)
    int sectionid;              // id of section containing item
    int itemid;                 // id of item within section

    // type-specific properties
    union
    {
        // check and button-related
        struct
        {
            int modifier;       // 0: none, 1: control, 2: shift; 4: alt
            int shortcut;       // shortcut key; 0: undefined
        } single;

        // static, radio and select-related
        struct
        {
            int nelem;          // number of elements in group
            char name[mjMAXUIMULTI][mjMAXUINAME]; // element names
        } multi;

        // slider-related
        struct
        {
            double range[2];    // slider range
            double divisions;   // number of range divisions
        } slider;

        // edit-related
        struct
        {
            int nelem;          // number of elements in list
            double range[mjMAXUIEDIT][2]; // element range (min>=max: ignore)
        } edit;
    };

    // internal
    mjrRect rect;               // rectangle occupied by item
};
typedef struct _mjuiItem mjuiItem;


struct _mjuiSection             // UI section
{
    // properties
    char name[mjMAXUINAME];     // name
    int state;                  // 0: closed, 1: open
    int modifier;               // 0: none, 1: control, 2: shift; 4: alt
    int shortcut;               // shortcut key; 0: undefined
    int nitem;                  // number of items in use
    mjuiItem item[mjMAXUIITEM];// preallocated array of items

    // internal
    mjrRect rtitle;             // rectangle occupied by title
    mjrRect rcontent;           // rectangle occupied by content
};
typedef struct _mjuiSection mjuiSection;


struct _mjUI                    // entire UI
{
    // constants set by user
    mjuiThemeSpacing spacing;   // UI theme spacing
    mjuiThemeColor color;       // UI theme color
    mjfItemEnable predicate;    // callback to set item state programmatically
    void* userdata;             // pointer to user data (passed to predicate)
    int rectid;                 // index of this ui rectangle in mjuiState
    int auxid;                  // aux buffer index of this ui
    int radiocol;               // number of radio columns (0 defaults to 2)

    // UI sizes (framebuffer units)
    int width;                  // width
    int height;                 // current heigth
    int maxheight;              // height when all sections open
    int scroll;                 // scroll from top of UI

    // mouse focus
    int mousesect;              // 0: none, -1: scroll, otherwise 1+section
    int mouseitem;              // item within section
    int mousehelp;              // help button down: print shortcuts

    // keyboard focus and edit
    int editsect;               // 0: none, otherwise 1+section
    int edititem;               // item within section
    int editcursor;             // cursor position
    int editscroll;             // horizontal scroll
    char edittext[mjMAXUITEXT]; // current text
    mjuiItem* editchanged;      // pointer to changed edit in last mjui_event

    // sections
    int nsect;                  // number of sections in use
    mjuiSection sect[mjMAXUISECT];  // preallocated array of sections
};
typedef struct _mjUI mjUI;


struct _mjuiDef
{
    int type;                   // type (mjtItem); -1: section
    char name[mjMAXUINAME];     // name
    int state;                  // state
    void* pdata;                // pointer to data
    char other[mjMAXUITEXT];    // string with type-specific properties
};
typedef struct _mjuiDef mjuiDef;
