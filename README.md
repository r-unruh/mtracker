# mtracker
cli movie tracker written in Rust - keep track of watched movies and series

## What's this?
mtracker is a simple cli tool that lets you keep track of watched movies and
series.

* Designed to work well with standard Linux command line tools like grep.
* Flat file system: All data is saved in a human-readable text file.
* No built-in cloud synchronization. Of course, you can set up some kind of
  synchronization yourself if you wish to.
* No data is fetched from the internet. You enter all the information that's
  useful to you manually.


## Installation
If you have Rust installed, you can simply use cargo:
```bash
cargo install mtracker
```

Otherwise just download the latest
[release](https://github.com/r-unruh/mtracker/releases) and put it somewhere
within your PATH variable, e.g.: `/usr/local/bin`

Don't forget to make the file executable:
```bash
sudo chmod +x /usr/local/bin/mtracker
```

More user-friendly install options will be added at some point.


## Tutorial
Let's assume your friend Max tells you about a fun horror movie. This is how
you add it to your watchlist:
```bash
mtracker add "Pearl (2022)" --tag=watchlist --note="Recommended by Max"
```

After watching the movie you decide to rate it a 8/10:
```bash
mtracker rate "Pearl (2022)" 8
```

This command assumes that you have now watched the item and removes it from the
watchlist automatically.

You can rate movies you already know directly without having to add them first:
```bash
mtracker rate "Session 9 (2001)" 10
mtracker rate "In Fabric (2018)" 4
```

Now lets see what we have so far by listing all items:
```bash
mtracker ls
```

Which returns this list, sorted by rating:
```bash
++++++++++ Session 9 (2001)
++++++++-- Pearl (2022)
+++------- In Fabric (2018)
```

This should cover the basics. Type `mtracker help [subcommand]` to see all
options.


## Database
The database is just a plain text file that you can edit by hand. It looks like
this:
```
Forrest Gump
year: 1994
rating: 9
tags: drama, comedy
last_seen: 2020-12-31

Bodies Bodies Bodies
year: 2022
tags: watchlist
note: recommended by Max

Whiplash
rating: 10
```

On Linux, the database file is automatically created and stored in:
`~/.local/share/mtracker/db.txt`


## Features
### Ratings
You can rate movies on a scale of your choice. mtracker doesn't force a rating
system. The highest rated item in your database determines the scale: If the
highest rated movie is a 7, then all the ratings go from 0 to 7. Of course, you
don't *have* to rate anything at all.

Here are a few options:
* 1 to 10: Rate the way that most websites do.
* 1 to 5: In case you prefer fewer options. No decimal numbers though.
* 0 to 1: Binary mode, or: Like/Dislike. Simple! Ratings don't have to start at
  1.
* 0 to 2: My personal favorite:
  * 2 = Like
  * 1 = Okayish
  * 0 = Dislike

### Tags
You can tag movies and filter by tags when listing them later. `watchlist` is a
special tag that highlights items and puts them on top of everything else.


## Command examples
Command                                               | Action
------------------------------------------------------|--------------
`mtracker ls`                                         | List all items
`mtracker ls --tag=horror,comedy`                     | List all items that are tagged both horror and comedy
`mtracker add "Aliens (1986)" --tag=watchlist,horror` | Add new item with tags OR add tags to an existing item
`mtracker rate "Aliens (1986)" 5`                     | Rate item a 5 (and remove from watchlist)
`mtracker ls \| grep -i aliens`                       | Use grep to find entries
`mtracker ls \| grep +++`                             | Use grep to search for items with a rating of at least 3
