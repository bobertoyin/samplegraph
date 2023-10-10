import React from "react";
import { useRouteError } from "react-router-dom";
import { Heading, Element, Button, Block } from "react-bulma-components";
import { Link } from "react-router-dom";

export default function ErrorPage(): React.ReactElement {
  const error: any = useRouteError();
  const error_msg = (
    <>
      {error.status ? (
        <Element display="inline" textColor="success">
          {`${error.status}`}{" "}
        </Element>
      ) : null}
      {`${error.statusText || error.message}`}
    </>
  );

  return (
    <Block textAlign="center">
      <Heading textColor="white" size={4}>
        {error_msg}
      </Heading>
      <Link to="/">
        <Button color="success" outlined radiusless>
          Return Home
        </Button>
      </Link>
    </Block>
  );
}
