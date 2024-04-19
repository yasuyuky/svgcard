import argparse


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('--font', default='Avenir Next', help='font name')
    parser.add_argument('--fontsize', default=10, type=int, help='font size')
    parser.add_argument('--xpitch', default=40, type=int, help='x pitch')
    parser.add_argument('--ypitch', default=30, type=int, help='y pitch')
    return parser


def main(font, fontsize, xpitch, ypitch):
    width = xpitch * 8
    height = ypitch * 7

    headers = f'''
[fontset]
yearmonth = ["{font} Bold"]
days = ["{font} Bold"]
normal = ["{font}"]

[dimension]
bezel = 0
height = {height}
width = {width}

[texts.year]
text = "{{year}}"
pos = [{xpitch},{ypitch//2}]
fontsize = {round(fontsize*2)}
fontset = "yearmonth"

[texts.month]
text = "{{month}}"
pos = [{xpitch+fontsize*6},{ypitch//2}]
fontsize = {round(fontsize*2)}
fontset = "yearmonth"

    '''
    print(headers)

    for i, d in enumerate(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]):
        print(f'[texts.{d}]')
        print(f'text = "{d.lower()}"')
        print(f'pos = [{i*xpitch+round(xpitch*0.75)}, {round(ypitch*1.5)}]')
        print('align = "left"')
        print(f'fontsize = {fontsize}')
        print('fontset = "days"')
        print()

    for i in range(5):
        for j in range(7):
            print(f'[texts.d{i}{j}]')
            print(f'text = "{{d{i}{j}}}"')
            print(f'pos = [{j*xpitch+round(xpitch*0.75)}, {round(i*ypitch+ypitch*2.5)}]')
            print('align = "left"')
            print(f'fontsize = {fontsize}')
            print('fontset = "normal"')
            print()


if __name__ == '__main__':
    parser = create_parser()
    args = parser.parse_args()
    main(args.font, args.fontsize, args.xpitch, args.ypitch)
