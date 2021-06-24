import React, { useEffect, useMemo, useRef, useState } from "react";
import { Autocomplete } from "@material-ui/lab";
import { Canvas, clearCanvas } from "./canvas";
import { CircularProgress, TextField } from "@material-ui/core";
import { debounce } from "util/input";
import { getTilings, handleEvent, setTiling, textSearch } from "client";
import styles from "./styles.module.scss";

const isMobile = () => window.outerWidth > 800;

const canvasWidth = isMobile()
  ? window.outerWidth - 100
  : window.outerWidth;
const canvasHeight = isMobile()
  ? window.outerHeight - 150
  : window.outerHeight - 120;

type TextSearchItem = {
  title: string;
  labels: { content: string }[];
};

export const App = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);

  const [tilings, setTilings] = useState([]);
  const [selected, setSelected] = useState("");
  const [searchOptions, setSearchOptions] = useState<TextSearchItem[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);

  const onSearchChange = useMemo(
    () => debounce(async (event: Event) => {
      // @ts-ignore
      const search = event.target.value;
      if (search === "") {
        setSearchOptions([]);
      } else {
        setSearchLoading(true);
        setSearchOptions(await textSearch(search));
        setSearchLoading(false);
      }
    }, 100),
    [],
  );

  useEffect(() => {
    (async () => {
      const tilings = await getTilings(null, null, null);
      setTilings(tilings);
    })();
  }, []);

  return (
    <>
      <h1 className={styles.title}>Tilings</h1>
      <select
        ref={selectRef}
        name="Tiling"
        className={styles.select}
        onChange={event => {
          // @ts-ignore
          const selected = event?.target?.value || 1;
          const canvas = canvasRef.current;
          setSelected(selected);

          if (canvas) {
            clearCanvas(canvas, true);
            setTiling(canvas, selected);
          }
        }}
      >
        <option value="">-</option>
        {tilings.map(({ id, title }) => (
          <option value={id} selected={title === selected}>{title}</option>
        ))}
      </select>
      <Autocomplete
        getOptionLabel={
          ({ title, labels }) => `${
            title
          }${
            labels.length === 0
              ? ""
              : ` - ${labels.map(({ content }) => content).join(", ")}`
          }`
        }
        loading={searchLoading}
        onInputChange={onSearchChange}
        options={searchOptions}
        renderInput={(params) => (
          <TextField
            {...params}
            label="Search"
            variant="outlined"
            InputProps={{
              ...params.InputProps,
              endAdornment: (
                <React.Fragment>
                  {searchLoading ? <CircularProgress color="inherit" size={20} /> : null}
                  {params.InputProps.endAdornment}
                </React.Fragment>
              ),
            }}
          />
        )}
        style={{ width: 300 }}
      />
      <Canvas
        ref={canvasRef}
        height={canvasHeight}
        width={canvasWidth}
        onMouseMove={handleEvent}
        onTouchMove={handleEvent}
      />
    </>
  );
};
