
### Main entry point

is `./page.js` (relative to this file)
it contains the `Home` component, do not modify heavily
only add other components if needed

the layout of the page is in `./layout.js` but do not modify this
the page css is defined in `./page..module.css`

#### Important Resources 

[Next Js Docs](https://nextjs.org/docs/app/building-your-application/routing)

## monitoringView

The view of the monitoring dashboard it contains `graph` and `genericTable` components
This also houses the buttons that change the Tables shown whether connections, remote_addresses or processes

### structure 
- layout.js 
    - defines the layout of the component 

- layout.module.css
    - defines the css styles of the component

## genericTable

A generic table that dynamically changes size and automatically sets its headers according to data given
see its usage in `monitoringView` component

### structure 
- layout.js 
    - defines the layout of the component 

- layout.module.css
    - defines the css styles of the component

## graph

graph component yet to be done


## globals.css

define the global styles used throughout the project
The `:root` is the big parent of all the HTML this is where we define our global css variables accessible by the `var()` keyword



---

## How to use components

see how `genericTable` and `Graph` are used in `monitoringView`

## How to style

always follow the styling found in other css files, if you are iffy about something just copy its css style from another file

## How to use images

see how images are imported and used in the main page in `src/app/page.js` or `monitoringView`

to add a new image but it in the `public` directory at the root of the project then use as it is used in the aforementioned files



