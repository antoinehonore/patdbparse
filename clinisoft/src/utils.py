import unidecode
import pandas as pd
import numpy as np


def pressure_sel(var):
    """Finds the pressure related variables."""
    return any([x in var for x in ["pfi", "pmean", "pinsp", "luftv", "peep", "ppeak", "luftv_tr_med",\
                            "luftv_trp_u_", "hfo_ampl"]]) or var == "cpap"


def append_clinid(d, clinid):
    """Append the integer clinid on the left hand side of dataFrame d with column name 'clinisoftid'."""
    out = pd.concat(
        [pd.DataFrame(index=d.index, columns=["clinid"], data=clinid * np.ones((d.shape[0])).astype(int)), d
         ], axis=1)
    return out


def format_fname(fname_out, thetype):
    return fname_out.replace(".csv", "_{}.csv".format(thetype))


def format_str(s):
    s_c = ''.join(e if e.isalnum() else '_' for e in s.replace(" ", "__"))
    out = unidecode.unidecode(s_c).lower()
    return out
