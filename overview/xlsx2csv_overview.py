import pandas as pd
import unittest

from pandas import read_excel
import argparse
import sys
import numpy as np

parser = argparse.ArgumentParser(description='Convert xlsx file into csv')
parser.add_argument('file_path', metavar='fname', type=str, nargs=1,
                    help='Input Excel file')

parser.add_argument('-sheetname', metavar='sheetname', type=str, nargs=1,
                    help='Specify sheetname', default=0)
parser.add_argument('-o', metavar='Output', type=str, nargs=1,
                    help='Output file', default=None)


class format_clinID(unittest.TestCase):
    def test_1(self):
        self.assertEqual(format_clinID("12345"), "12345")

    def test_2(self):
        self.assertEqual(format_clinID("12345 or 56789"), "12345 or 56789")

    def test_3(self):
        self.assertEqual(format_clinID("dfsadg"), np.nan)


def format_num_column(x):
    if isinstance(x, str):
        x = x.replace(",", ".")
        if x.replace(".", "").isdigit():
            return float(x)
        else:
            return np.nan
    return x


def format_clinID(x):
    """ """
    if isinstance(x, int):
        return str(x)

    elif isinstance(x, float):
        if np.isnan(x):
            return x
        else:
            return str(int(x))

    elif isinstance(x, str) and all(xx.isdigit() for xx in x.split(" or ")):
        return x

    else:
        return np.nan


if __name__ == "__main__":
    args = parser.parse_args()
    infile = args.file_path[0]
    data = read_excel(infile, sheet_name=args.sheetname, index_col=0)

    # data[['Recruited by', 'TakeCare Harvest', 'TakeCare Review']] \
    #    = data[['Recruited by', 'TakeCare Harvest', 'TakeCare Review']].fillna('-')

    critical_var = ['ClinisoftID', 'Personnummer', 'Sex', 'BirthDate', 'BW', 'GA(W)']
    non_critical_var = ['Recruited by', 'TakeCare Harvest', 'TakeCare Review',
                        'apgar 1', 'apgar 5', 'apgar 10', 'Delivery', "Head circum"]

    varnames = critical_var + non_critical_var

    # Fix some columns problems
    data["Head circum"] = data["Head circum"].apply(format_num_column)
    data["ClinisoftID"] = data["ClinisoftID"].apply(format_clinID)

    # Limit the columns loaded in the database
    data = data[varnames]

    # Drop the rows with nan values on critical info, not this info:
    data[non_critical_var] = data[non_critical_var].fillna('-')

    # Drop nans on critical info
    data = data.dropna()

    # Put nan back in to the frame
    data.replace(to_replace='-', value=np.nan, inplace=True)

    if args.o is None:
        outfile = infile.replace(".xlsx", ".csv")
    else:
        if args.o[0] == '-':
            outfile = sys.stdout
        else:
            outfile = args.o[0]

    data.to_csv(outfile, sep=";", encoding='utf8')
    sys.exit(0)

