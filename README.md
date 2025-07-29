# Timetable scraper for custom frontends

This is a rust rewrite based on plojyon's project.

[Github Link](https://github.com/plojyon/timetable_scraper/tree/master)

## Endpoints

```
/ root
/timetable/{uni}/{group}
```

## Response format
```json
[
  {
    "day": 0,
    "time": 9,
    "duration": 2,
    "professor": "Tim Oblak",
    "classroom": "PR15",
    "subject": {
      "name": "OS(63709)_LV",
      "abbreviation": "OS(63709)_LV",
      "location": "FRI",
      "type": " LV"
    }
  },
  {
    "day": 0,
    "time": 15,
    "duration": 2,
    "professor": "Bojan Klemenc",
    "classroom": "PR05",
    "subject": {
      "name": "OS(63709)_LV",
      "abbreviation": "OS(63709)_LV",
      "location": "FRI",
      "type": " LV"
    }
  },
]
```

Currently supported:

- fri

**Feel free to make a PR of your own!**
