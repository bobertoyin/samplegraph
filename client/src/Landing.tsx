import React, { useState } from "react";
import { Link } from "react-router-dom";
import {
  Heading,
  Element,
  Form,
  Icon,
  Notification,
} from "react-bulma-components";
import { RandomReveal } from "react-random-reveal";
import { debounce } from "lodash";

import { SearchResponse } from "./bindings/SearchResponse";

function JumbledWord(): React.ReactElement {
  return (
    <Element textColor="success" display="inline">
      <RandomReveal
        isPlaying
        duration={3}
        updateInterval={0.03}
        characters="samples"
        onComplete={() => ({ shouldRepeat: true, delay: 2 })}
      />
    </Element>
  );
}

async function get_search(
  query: string,
  setter: React.Dispatch<React.SetStateAction<SearchResponse>>,
  errorSetter: React.Dispatch<React.SetStateAction<string | undefined>>,
  loadSetter: React.Dispatch<React.SetStateAction<boolean>>,
): Promise<void> {
  loadSetter(true);
  if (query === "") {
    setter({ hits: [] });
    errorSetter(undefined);
  } else {
    const response = await fetch(`/api/search?query=${query}`);
    if (!response.ok) {
      errorSetter(await response.text());
    } else {
      const json: SearchResponse = await response.json();
      setter(json);
    }
  }
  loadSetter(false);
}

function SearchResults(props: {
  results: SearchResponse;
  error: string | undefined;
}): React.ReactElement | null {
  const { results, error } = props;
  if (error) {
    return (
      <ul>
        <li style={{ listStyle: "none" }}>{error.toUpperCase()}</li>
      </ul>
    );
  }
  if (results.hits.length > 0) {
    let items = results.hits.map((result) => {
      return (
        <li key={result.id}>
          <Link to={`/graph/${result.id}`}>{result.full_title}</Link>
        </li>
      );
    });
    return <ul>{items}</ul>;
  }
  return null;
}

function Search(): React.ReactElement {
  const [results, setResults] = useState<SearchResponse>({ hits: [] });
  const [error, setError] = useState<string>();
  const [loading, setLoading] = useState(false);
  return (
    <form onSubmit={(e: React.FormEvent) => e.preventDefault()}>
      <Form.Field>
        <Form.Control fullwidth loading={loading}>
          <Icon align="left">
            <i className="material-symbols-outlined">search</i>
          </Icon>
          <Form.Input
            placeholder="Search"
            radiusless
            color="white"
            backgroundColor="black"
            textColor="white"
            onChange={debounce(
              (e: React.ChangeEvent<HTMLInputElement>) =>
                get_search(
                  e.target.value.trim(),
                  setResults,
                  setError,
                  setLoading,
                ),
              200,
            )}
          />
          <div className="search-results">
            <SearchResults results={results} error={error} />
          </div>
        </Form.Control>
      </Form.Field>
    </form>
  );
}

export default function Landing(): React.ReactElement {
  return (
    <>
      <Heading textColor="white" size={2}>
        SampleGraph
      </Heading>
      <Heading textColor="white" size={5} subtitle>
        Visualize the relationship between songs by their <JumbledWord />.
      </Heading>
      <Search />
    </>
  );
}
