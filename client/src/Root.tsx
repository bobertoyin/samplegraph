import React, { PropsWithChildren } from "react";
import { Hero, Content, Section, Container } from "react-bulma-components";
import { Outlet } from "react-router-dom";

export default function Root(props: PropsWithChildren<{}>): React.ReactElement {
  return (
    <Hero size="fullheight" backgroundColor="black" textColor="success">
      <Hero.Header></Hero.Header>
      <Hero.Body justifyContent="center">
        <Section>
          <Container>
            <Content>{props.children ?? <Outlet />}</Content>
          </Container>
        </Section>
      </Hero.Body>
      <Hero.Footer></Hero.Footer>
    </Hero>
  );
}
