# mtracker
mtracker is a simple cli tool for Linux that lets you keep track of watched
movies and series.

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
```bash
sudo curl -o /usr/local/bin/mtracker https://github.com/r-unruh/mtracker/releases/latest/download/mtracker
```

Don't forget to make the file executable:
```bash
sudo chmod +x /usr/local/bin/mtracker
```

More user-friendly install options will probably be added at some point.


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

> [!NOTE]
> Commands are not yet stable and may change in future versions.
> Make sure to backup your database on a regular basis.


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

On Linux, the database file is automatically created and stored in `~/.local/share/mtracker/db.txt`. If any relevant XDG environment variables (e.g., `XDG_DATA_HOME`) are set, they will be respected, and the file will be stored according to the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/).

## Features
### Ratings
You can rate movies on a scale of your choice. mtracker doesn't force a rating
system. The highest rated item in your database determines the scale: If the
highest rated movie is a 7, then all the ratings go from 0 to 7. Of course, you
don't *have* to rate anything at all.

Here are a few options:

<table>
  <tr>
    <th>Rating Scale</th>
    <th>Explanation</th>
  </tr>
  <tr>
    <td>1 to 10</td>
    <td>You can rate the way most movie websites do.</td>
  </tr>
  <tr>
    <td>1 to 5</td>
    <td>
      In case you prefer fewer options, this might be better. There are no
      decimal numbers though.
    </td>
  </tr>
  <tr>
    <td>0 to 1</td>
    <td>
      Binary mode, or: Like/Dislike. Most simple! Ratings don't have to start
      at 1.
    </td>
  </tr>
  <tr>
    <td>0 to 2</td>
    <td>
      <p>
        If you often find yourself neither liking nor disliking movies, you may
        need a third option. This is the system I'm using:
      </p>
      <ul>
        <li>2 = Like</li>
        <li>1 = Okayish</li>
        <li>0 = Dislike</li>
      </ul>
    </td>
  </tr>
</table>

### Tags
You can tag movies and filter by tags when listing them later. `watchlist` is a
special tag that highlights items and puts them on top of everything else.

### Special search terms
When listing itims with the `ls` subcommand, you can filter for additional attributes:

Term                | Meaning
--------------------|--------------
`rated`             | List items that have a rating
`unrated`           | List items without a rating
`<year>`            | List items released in `<year>` (see examples)
`<year>-<year>`     | List items released between `<year>` and `<year>`
`-<year>`           | List items released before or in `<year>`
`<year>-`           | List items released after or in `<year>`


## Command examples
Command                                               | Action
------------------------------------------------------|--------------
`mtracker ls`                                         | List all items
`mtracker ls horror comedy`                           | List items that are tagged both horror and comedy
`mtracker ls horror 2022-2024"`                       | List horror movies that were released between 2022 and 2024
`mtracker add "Aliens (1986)" --tag=watchlist,horror` | Add new item with tags OR add tags to an existing item
`mtracker rate "Aliens (1986)" 5`                     | Rate item a 5 (and remove from watchlist)
`mtracker ls \| grep -i aliens`                       | Use grep to find entries
`mtracker ls \| grep +++`                             | Use grep to search for items with a rating of at least 3
