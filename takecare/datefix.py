
import pandas as pd
import argparse
parser=argparse.ArgumentParser()
parser.add_argument("-i", help="input takecare file", metavar="infile",required=True)

def format_date(s):
    d = pd.to_datetime(s)
    return d.strftime("%Y-%m-%d %H:%M:%S")


if __name__ == "__main__":

    args = parser.parse_args()
    infile = args.i
    df = pd.read_excel(infile)
    dates = df["Date"]
    dates_formatted = dates.apply(format_date)
    df["Date"] = dates_formatted
    df.to_excel(infile.replace(".xlsx","_new.xlsx"))