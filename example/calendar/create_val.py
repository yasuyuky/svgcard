import argparse
import calendar


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('year', type=int, help='year')
    parser.add_argument('month', type=int, help='month')
    return parser


def main(year, month):
    calendar.setfirstweekday(calendar.SUNDAY)
    cal = calendar.monthcalendar(year, month)
    print(f'year = "{year}"')
    print(f'month = "{calendar.month_abbr[month]}"')

    for i in range(5):
        for j in range(7):
            if cal[i][j]:
                print(f'd{i}{j} = "{cal[i][j]:02d}"')
            else:
                print(f'd{i}{j} = ""')


if __name__ == '__main__':
    parser = create_parser()
    args = parser.parse_args()
    main(args.year, args.month)
